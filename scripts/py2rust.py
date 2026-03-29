#!/usr/bin/env python3
"""py2rust — Convert OpenAI Python SDK types to Rust serde types.

Usage:
  py2rust.py sync <python_types_dir> <rust_crate_src_dir>
  py2rust.py sync <python_types_dir> <rust_crate_src_dir> --dry-run
  py2rust.py file <python_file>

Sync mode:
  Walks ALL subdirectories in <python_types_dir>, converts BaseModel classes,
  TypedDict classes, and TypeAlias definitions into Rust types.

  Generated code goes into _gen.rs files (machine-owned, overwritten on sync).
  Manual .rs files are never touched — types found in them are skipped.

  Override mechanism:
    1. Script scans all .rs files in target dir EXCEPT _gen.rs and mod.rs
    2. Extracts pub struct/enum names from those files
    3. Skips those types when generating _gen.rs
    4. mod.rs re-exports from _gen.rs + manual files

Examples:
  # Full sync — all domains
  python3 scripts/py2rust.py sync ~/openai-python/src/openai/types/ openai-oxide-types/src/

  # Dry run — show what would be generated
  python3 scripts/py2rust.py sync ~/openai-python/src/openai/types/ openai-oxide-types/src/ --dry-run

  # Single file to stdout
  python3 scripts/py2rust.py file ~/openai-python/src/openai/types/batch.py
"""

import ast
import re
import sys
import fnmatch
from pathlib import Path
from dataclasses import dataclass, field as datafield
from typing import Optional


# ── Python → Rust type mapping ──

TYPE_MAP: dict[str, str] = {
    "str": "String",
    "int": "i64",
    "float": "f64",
    "bool": "bool",
    "object": "serde_json::Value",
    "None": "()",
    "NoneType": "()",
    "bytes": "Vec<u8>",
    "dict": "serde_json::Value",
    "Dict": "serde_json::Value",
    "Any": "serde_json::Value",
    "Iterable": "Vec",  # Iterable[T] → Vec<T>
    "HttpxBinaryResponseContent": "Vec<u8>",
}

# Reserved Rust keywords that need renaming
RUST_KEYWORDS = {"type", "self", "super", "crate", "mod", "fn", "struct", "enum",
                 "trait", "impl", "pub", "use", "let", "mut", "ref", "match",
                 "if", "else", "loop", "while", "for", "in", "return", "break",
                 "continue", "move", "box", "where", "async", "await", "dyn",
                 "static", "const", "unsafe", "extern", "abstract", "final",
                 "override", "macro", "yield", "try", "become", "priv", "typeof",
                 "unsized", "virtual", "do"}


# ── Data model ──

@dataclass
class RustField:
    name: str           # original Python name
    rust_name: str      # Rust field name (snake_case, keyword-safe)
    rust_type: str      # Rust type string
    optional: bool      # wrapped in Option<>
    doc: Optional[str] = None
    rename: Optional[str] = None  # serde(rename = "...")
    alias: Optional[str] = None   # from Field(alias=...)


@dataclass
class RustStruct:
    name: str
    fields: list[RustField]
    doc: Optional[str] = None
    source_file: str = ""
    kind: str = "struct"  # "struct" or "request" (from TypedDict)


@dataclass
class RustEnum:
    name: str
    variants: list[tuple[str, Optional[str]]]  # (VariantName, serde_rename_or_None)
    doc: Optional[str] = None
    source_file: str = ""


@dataclass
class RustTypeAlias:
    name: str
    target: str
    doc: Optional[str] = None
    source_file: str = ""


RustItem = RustStruct | RustEnum | RustTypeAlias


# ── Domain routing — Python directory → Rust module ──

# Subdirectories map 1:1
SUBDIR_DOMAINS = {
    "responses", "chat", "audio", "beta", "fine_tuning",
    "realtime", "conversations", "containers", "evals",
    "graders", "shared", "skills", "uploads", "vector_stores",
    "webhooks",
}

# Top-level files are routed by prefix
# Top-level files route INTO existing subdir domains where possible
TOPLEVEL_ROUTES: list[tuple[str, str]] = [
    ("batch*", "batch"),
    ("completion*", "completion"),
    ("create_embedding*", "embedding"),
    ("embedding*", "embedding"),
    ("file_*", "file"),
    ("auto_file_*", "file"),
    ("static_file_*", "file"),
    ("other_file_*", "file"),
    ("image*", "image"),
    ("images_*", "image"),
    ("model*", "model"),
    ("chat_model*", "model"),
    ("audio_model*", "audio"),        # merge into audio/
    ("audio_response_format*", "audio"),
    ("moderation*", "moderation"),
    ("upload*", "uploads"),            # merge into uploads/
    ("vector_store*", "vector_stores"),  # merge into vector_stores/
    ("video*", "video"),
    ("eval_*", "evals"),               # merge into evals/
    ("container_*", "containers"),      # merge into containers/
    ("skill*", "skills"),              # merge into skills/
    ("deleted_skill*", "skills"),
    ("websocket*", "websocket"),
]


def route_toplevel_file(filename: str) -> str:
    """Route a top-level types/*.py file to a Rust module name."""
    for pattern, domain in TOPLEVEL_ROUTES:
        if fnmatch.fnmatch(filename, pattern):
            return domain
    return "extra"


# ── Parser ──

class Parser:
    """Parses Python type files and converts to Rust items."""

    def __init__(self):
        self.known_types: set[str] = set()
        # Maps original Python name → prefixed Rust name (for generic names)
        self.name_map: dict[str, str] = {}
        self._pending_enums: list[RustEnum] = []

    def prescan_file(self, path: Path, prefix: str = ""):
        """Pass 0: collect type names without generating code.

        This allows cross-file references within a domain to resolve
        to proper type names instead of serde_json::Value.
        Registers both original Python name AND prefixed Rust name.
        """
        source = path.read_text()
        try:
            tree = ast.parse(source)
        except SyntaxError:
            return

        generic_names = {"Content", "Part", "Summary", "Action", "Output", "Result",
                         "Tool", "Item", "Error", "Details", "Environment", "Operation",
                         "Outcome", "Errors", "Choice", "Function"}

        for node in ast.iter_child_nodes(tree):
            # TypeAlias
            if isinstance(node, ast.AnnAssign) and self._is_type_alias(node):
                if isinstance(node.target, ast.Name) and node.target.id:
                    self.known_types.add(node.target.id)
            # Assignment alias
            elif isinstance(node, ast.Assign) and len(node.targets) == 1:
                target = node.targets[0]
                if isinstance(target, ast.Name) and isinstance(node.value, ast.Subscript):
                    self.known_types.add(target.id)
            # Classes
            elif isinstance(node, ast.ClassDef):
                orig = node.name
                name = orig
                if prefix and name in generic_names:
                    name = f"{prefix}{name}"
                # Register BOTH original and prefixed name
                self.known_types.add(orig)
                self.known_types.add(name)
                if orig != name:
                    if orig in self.name_map and self.name_map[orig] != name:
                        # Ambiguous: same original name maps to different prefixed names
                        del self.name_map[orig]
                        self.known_types.discard(orig)  # prevent using bare name
                    else:
                        self.name_map[orig] = name
                # Also prescan nested classes
                for inner in node.body:
                    if isinstance(inner, ast.ClassDef):
                        inner_orig = inner.name
                        inner_name = inner_orig
                        if inner_name in generic_names:
                            inner_name = f"{name}{inner_name}"
                        self.known_types.add(inner_orig)
                        self.known_types.add(inner_name)
                        if inner_orig != inner_name:
                            if inner_orig in self.name_map and self.name_map[inner_orig] != inner_name:
                                del self.name_map[inner_orig]
                                self.known_types.discard(inner_orig)
                            else:
                                self.name_map[inner_orig] = inner_name

    def parse_file(self, path: Path, prefix: str = "") -> list[RustItem]:
        """Parse a single Python file, returning Rust items."""
        source = path.read_text()
        try:
            tree = ast.parse(source)
        except SyntaxError:
            return []

        items: list[RustItem] = []
        seen_in_file: set[str] = set()

        for node in ast.iter_child_nodes(tree):
            # TypeAlias = Literal[...] or TypeAlias = Union[...]
            if isinstance(node, ast.AnnAssign) and self._is_type_alias(node):
                item = self._parse_type_alias(node, str(path))
                if item and item.name not in seen_in_file:
                    seen_in_file.add(item.name)
                    self.known_types.add(item.name)
                    items.append(item)

            # Regular assignment: Foo = Literal[...] (older pattern)
            elif isinstance(node, ast.Assign) and len(node.targets) == 1:
                item = self._parse_assign_alias(node, str(path))
                if item and item.name not in seen_in_file:
                    seen_in_file.add(item.name)
                    self.known_types.add(item.name)
                    items.append(item)

            # Class definitions (BaseModel, TypedDict)
            elif isinstance(node, ast.ClassDef):
                cls_items = self._parse_class(node, str(path), prefix)
                for item in cls_items:
                    if item.name not in seen_in_file:
                        seen_in_file.add(item.name)
                        items.append(item)

        return items

    def _is_type_alias(self, node: ast.AnnAssign) -> bool:
        """Check if this is a TypeAlias annotation."""
        if isinstance(node.annotation, ast.Name) and node.annotation.id == "TypeAlias":
            return True
        if isinstance(node.annotation, ast.Attribute) and node.annotation.attr == "TypeAlias":
            return True
        return False

    def _parse_type_alias(self, node: ast.AnnAssign, source: str) -> Optional[RustItem]:
        """Parse: FooType: TypeAlias = Literal["a", "b"] or Union[...]."""
        if not isinstance(node.target, ast.Name) or node.value is None:
            return None
        name = node.target.id
        return self._convert_alias_value(name, node.value, source)

    def _parse_assign_alias(self, node: ast.Assign, source: str) -> Optional[RustItem]:
        """Parse: FooType = Literal["a", "b"] (without TypeAlias annotation)."""
        target = node.targets[0]
        if not isinstance(target, ast.Name):
            return None
        name = target.id
        # Only handle if value looks like a type (Literal, Union, Annotated)
        if not isinstance(node.value, ast.Subscript):
            return None
        origin = node.value.value
        if isinstance(origin, ast.Name) and origin.id in ("Literal", "Union", "Annotated"):
            return self._convert_alias_value(name, node.value, source)
        return None

    def _convert_alias_value(self, name: str, value: ast.expr, source: str) -> Optional[RustItem]:
        """Convert a type alias value to a RustEnum or RustTypeAlias."""
        if not name or not name.strip():
            return None
        # Literal["a", "b", "c"] → enum
        if isinstance(value, ast.Subscript) and isinstance(value.value, ast.Name):
            if value.value.id == "Literal":
                variants = self._extract_literal_variants(value)
                if variants:
                    return RustEnum(
                        name=name,
                        variants=variants,
                        source_file=source,
                    )
            # Optional[Literal["a", "b"]] → enum (not Option<enum>)
            elif value.value.id == "Optional":
                inner = value.slice
                if (isinstance(inner, ast.Subscript) and isinstance(inner.value, ast.Name)
                        and inner.value.id == "Literal"):
                    variants = self._extract_literal_variants(inner)
                    if variants:
                        return RustEnum(
                            name=name,
                            variants=variants,
                            source_file=source,
                        )
            # Union[A, B, C] → check if all Literals → enum, else type alias
            elif value.value.id == "Union":
                return self._convert_union_alias(name, value, source)
            # Annotated[Union[...], PropertyInfo(discriminator="type")]
            elif value.value.id == "Annotated":
                return self._convert_annotated_alias(name, value, source)

        # Dict[str, str] → type alias
        rust_type = self._convert_type(value)
        if rust_type and rust_type != "serde_json::Value":
            return RustTypeAlias(name=name, target=rust_type, source_file=source)
        # Still emit a Value alias so the name is defined (prevents compile errors)
        return RustTypeAlias(name=name, target="serde_json::Value", source_file=source)

    def _convert_union_alias(self, name: str, node: ast.Subscript, source: str) -> Optional[RustItem]:
        """Convert Union[A, B, C] to enum or type alias."""
        if not isinstance(node.slice, ast.Tuple):
            return None

        # Check if all elements are literal strings
        all_literals: list[str] = []
        for elt in node.slice.elts:
            if isinstance(elt, ast.Subscript) and isinstance(elt.value, ast.Name) and elt.value.id == "Literal":
                variants = self._extract_literal_values(elt)
                all_literals.extend(variants)
            elif isinstance(elt, ast.Constant) and isinstance(elt.value, str):
                all_literals.append(elt.value)

        if all_literals:
            variants = [(self._value_to_variant(v), v if self._value_to_variant(v).lower() != v else None)
                        for v in all_literals]
            return RustEnum(name=name, variants=variants, source_file=source)

        # Mixed union — just a type alias to serde_json::Value
        return RustTypeAlias(name=name, target="serde_json::Value", source_file=source)

    def _convert_annotated_alias(self, name: str, node: ast.Subscript, source: str) -> Optional[RustItem]:
        """Convert Annotated[Union[...], PropertyInfo(discriminator="type")] → tagged enum."""
        if not isinstance(node.slice, ast.Tuple) or len(node.slice.elts) < 1:
            return None
        inner = node.slice.elts[0]
        # Delegate to the inner Union
        if isinstance(inner, ast.Subscript) and isinstance(inner.value, ast.Name) and inner.value.id == "Union":
            # Extract variant type names
            if isinstance(inner.slice, ast.Tuple):
                variant_names = []
                for elt in inner.slice.elts:
                    if isinstance(elt, ast.Name):
                        variant_names.append(elt.id)
                    elif isinstance(elt, ast.Attribute):
                        variant_names.append(elt.attr)
                if variant_names:
                    # Create type alias for now (proper tagged enum is complex)
                    return RustTypeAlias(
                        name=name,
                        target="serde_json::Value",
                        source_file=source,
                    )
        return None

    def _parse_class(self, cls: ast.ClassDef, source: str, prefix: str = "") -> list[RustItem]:
        """Parse a class definition (BaseModel or TypedDict)."""
        base_kind = self._classify_class(cls)
        if base_kind is None:
            return []

        # Collect items: the main struct + any inline enums
        self._pending_enums.clear()

        # Resolve name with prefix for generic names
        name = cls.name
        generic_names = {"Content", "Part", "Summary", "Action", "Output", "Result",
                         "Tool", "Item", "Error", "Details", "Environment", "Operation",
                         "Outcome", "Errors", "Choice", "Function"}
        if prefix and name in generic_names:
            name = f"{prefix}{name}"

        doc = self._extract_docstring(cls)
        fields = self._extract_fields(cls, name, is_typeddict=(base_kind == "typeddict"))
        struct = RustStruct(
            name=name,
            fields=fields,
            doc=doc,
            source_file=source,
            kind="request" if base_kind == "typeddict" else "struct",
        )

        items: list[RustItem] = []
        # Add inline enums generated from Literal fields
        # (names already in known_types from _convert_literal — that's fine,
        #  just collect them unconditionally; dedup happens at file/domain level)
        items.extend(self._pending_enums)
        self._pending_enums.clear()

        items.append(struct)

        # Parse nested classes (inner BaseModel/TypedDict)
        for node in cls.body:
            if isinstance(node, ast.ClassDef):
                nested_items = self._parse_class(node, source, prefix=name)
                items.extend(nested_items)

        return items

    def _classify_class(self, cls: ast.ClassDef) -> Optional[str]:
        """Classify class as 'basemodel', 'typeddict', or None."""
        for base in cls.bases:
            name = self._base_name(base)
            if name in ("BaseModel",):
                return "basemodel"
            if name in ("TypedDict",):
                return "typeddict"
        # Check keyword bases: class Foo(TypedDict, total=False)
        for kw in cls.keywords:
            pass  # keywords don't change base classification
        for base in cls.bases:
            name = self._base_name(base)
            if name and "TypedDict" in name:
                return "typeddict"
        return None

    def _base_name(self, base: ast.expr) -> Optional[str]:
        if isinstance(base, ast.Name):
            return base.id
        if isinstance(base, ast.Attribute):
            return base.attr
        return None

    def _extract_docstring(self, cls: ast.ClassDef) -> Optional[str]:
        if (cls.body and isinstance(cls.body[0], ast.Expr)
                and isinstance(cls.body[0].value, ast.Constant)
                and isinstance(cls.body[0].value.value, str)):
            return cls.body[0].value.value.strip().split("\n")[0]
        return None

    def _extract_fields(self, cls: ast.ClassDef, class_name: str,
                        is_typeddict: bool = False) -> list[RustField]:
        """Extract fields from class body."""
        fields = []
        for i, node in enumerate(cls.body):
            if not isinstance(node, ast.AnnAssign) or not node.target:
                continue

            py_name = node.target.id if isinstance(node.target, ast.Name) else str(node.target)
            annotation = node.annotation

            # Check for Required[T] wrapper (TypedDict)
            is_required = False
            if is_typeddict and isinstance(annotation, ast.Subscript):
                if isinstance(annotation.value, ast.Name) and annotation.value.id == "Required":
                    annotation = annotation.slice
                    is_required = True

            # Convert type
            rust_type = self._convert_type(annotation, field_name=py_name, class_name=class_name)

            # Determine optionality
            has_default_none = (node.value is not None and
                                isinstance(node.value, ast.Constant) and
                                node.value.value is None)
            optional = has_default_none or (is_typeddict and not is_required)

            if optional and not rust_type.startswith("Option<"):
                rust_type = f"Option<{rust_type}>"

            # Field doc: Python uses string expr after field assignment
            doc = self._field_docstring(cls.body, i)

            # Handle alias from Field(alias="...")
            alias = self._extract_field_alias(node.value)

            # Rust name
            rust_name = self._to_rust_field_name(py_name)
            rename = None
            if rust_name != py_name:
                rename = py_name
            if alias:
                rename = alias

            fields.append(RustField(
                name=py_name,
                rust_name=rust_name,
                rust_type=rust_type,
                optional=optional,
                doc=doc,
                rename=rename,
                alias=alias,
            ))

        return fields

    def _extract_field_alias(self, value_node) -> Optional[str]:
        """Extract alias from Field(alias="...") or FieldInfo(alias="...")."""
        if not isinstance(value_node, ast.Call):
            return None
        func = value_node.func
        if isinstance(func, ast.Name) and func.id in ("Field", "FieldInfo"):
            for kw in value_node.keywords:
                if kw.arg == "alias" and isinstance(kw.value, ast.Constant):
                    return str(kw.value.value)
        return None

    def _field_docstring(self, body: list[ast.stmt], idx: int) -> Optional[str]:
        """Check if statement after body[idx] is a docstring."""
        if idx + 1 < len(body):
            next_stmt = body[idx + 1]
            if (isinstance(next_stmt, ast.Expr)
                    and isinstance(next_stmt.value, ast.Constant)
                    and isinstance(next_stmt.value.value, str)):
                return next_stmt.value.value.strip().split("\n")[0]
        return None

    # ── Type conversion ──

    def _convert_type(self, node: ast.expr, optional: bool = False,
                      field_name: str = "", class_name: str = "") -> str:
        """Convert Python type annotation to Rust type string."""
        if isinstance(node, ast.Constant):
            return TYPE_MAP.get(str(node.value), "String")

        if isinstance(node, ast.Name):
            name = node.id
            if name in TYPE_MAP:
                return TYPE_MAP[name]
            # Resolve prefixed name (e.g. Choice → ChatCompletionChoice)
            resolved = self.name_map.get(name, name)
            if resolved in self.known_types:
                return resolved
            if name in self.known_types:
                return name
            # Check if it's a builtins type we should map
            if name == "Mapping":
                return "serde_json::Map<String, serde_json::Value>"
            # Unknown type → Value (compiles, can be refined later)
            return "serde_json::Value"

        if isinstance(node, ast.Attribute):
            return "serde_json::Value"

        if isinstance(node, ast.Subscript):
            return self._convert_subscript(node, field_name, class_name)

        # T | None → Option<T>
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.BitOr):
            return self._convert_bitor(node, field_name, class_name)

        return "serde_json::Value"

    def _convert_subscript(self, node: ast.Subscript, field_name: str = "",
                           class_name: str = "") -> str:
        """Convert subscript types: Optional[T], List[T], Dict[K,V], Literal[...], etc."""
        origin = node.value
        if not isinstance(origin, ast.Name):
            return "serde_json::Value"

        name = origin.id

        if name == "Optional":
            inner = self._convert_type(node.slice, field_name=field_name, class_name=class_name)
            return f"Option<{inner}>"

        if name in ("List", "Sequence", "Iterable"):
            inner = self._convert_type(node.slice)
            if not inner or inner == "serde_json::Value":
                return "Vec<serde_json::Value>"
            return f"Vec<{inner}>"

        if name == "Dict":
            if isinstance(node.slice, ast.Tuple) and len(node.slice.elts) == 2:
                key = self._convert_type(node.slice.elts[0])
                val = self._convert_type(node.slice.elts[1])
                if key == "String":
                    return f"std::collections::HashMap<String, {val}>"
            return "serde_json::Value"

        if name == "Mapping":
            if isinstance(node.slice, ast.Tuple) and len(node.slice.elts) == 2:
                key = self._convert_type(node.slice.elts[0])
                val = self._convert_type(node.slice.elts[1])
                if key == "String":
                    return f"std::collections::HashMap<String, {val}>"
            return "serde_json::Value"

        if name == "Literal":
            return self._convert_literal(node, field_name, class_name)

        if name == "Union":
            return self._convert_union(node)

        if name == "Annotated":
            if isinstance(node.slice, ast.Tuple) and node.slice.elts:
                return self._convert_type(node.slice.elts[0], field_name=field_name, class_name=class_name)
            return self._convert_type(node.slice)

        if name == "Required":
            return self._convert_type(node.slice, field_name=field_name, class_name=class_name)

        if name == "Set":
            inner = self._convert_type(node.slice)
            return f"Vec<{inner}>"

        if name == "FrozenSet":
            inner = self._convert_type(node.slice)
            return f"Vec<{inner}>"

        if name == "Tuple":
            if isinstance(node.slice, ast.Tuple):
                types = [self._convert_type(e) for e in node.slice.elts]
                return f"({', '.join(types)})"
            return f"({self._convert_type(node.slice)},)"

        return "serde_json::Value"

    def _convert_bitor(self, node: ast.BinOp, field_name: str = "",
                       class_name: str = "") -> str:
        """Convert T | None to Option<T>."""
        left = self._convert_type(node.left, field_name=field_name, class_name=class_name)
        right = self._convert_type(node.right, field_name=field_name, class_name=class_name)
        if right in ("()", "None"):
            return f"Option<{left}>"
        if left in ("()", "None"):
            return f"Option<{right}>"
        # T | U — untagged enum, use Value for now
        return "serde_json::Value"

    def _convert_literal(self, node: ast.Subscript, field_name: str, class_name: str) -> str:
        """Convert Literal["a", "b"] — single value → String, multi → enum."""
        values = self._extract_literal_values(node)
        if not values:
            return "String"

        # Single value = type discriminator, use String
        if len(values) == 1:
            return "String"

        # Multiple values → generate inline enum
        enum_name = self._literal_enum_name(field_name, class_name)
        if enum_name in self.known_types:
            return enum_name

        variants = []
        for v in values:
            variant = self._value_to_variant(str(v))
            # Always rename if variant text differs from original value
            rename = str(v) if variant != str(v) else None
            variants.append((variant, rename))

        enum = RustEnum(name=enum_name, variants=variants)
        self._pending_enums.append(enum)
        self.known_types.add(enum_name)
        return enum_name

    def _convert_union(self, node: ast.Subscript) -> str:
        """Convert Union[A, B, ...] inline."""
        if not isinstance(node.slice, ast.Tuple):
            return "serde_json::Value"

        types = []
        for elt in node.slice.elts:
            t = self._convert_type(elt)
            if t not in ("()", "None", ""):
                types.append(t)

        if not types:
            return "serde_json::Value"
        if len(types) == 1:
            return f"Option<{types[0]}>"
        # Multiple non-None types — check if String + something else
        if len(types) == 2 and "String" in types:
            return "String"  # String can represent either
        return "serde_json::Value"

    # ── Helpers ──

    def _extract_literal_values(self, node: ast.Subscript) -> list:
        """Extract values from Literal["a", "b"]."""
        slice_node = node.slice
        if isinstance(slice_node, ast.Tuple):
            return [elt.value for elt in slice_node.elts
                    if isinstance(elt, ast.Constant)]
        if isinstance(slice_node, ast.Constant):
            return [slice_node.value]
        return []

    def _extract_literal_variants(self, node: ast.Subscript) -> list[tuple[str, Optional[str]]]:
        """Extract (VariantName, serde_rename) pairs from Literal[...]."""
        values = self._extract_literal_values(node)
        if not values or not all(isinstance(v, str) for v in values):
            return []
        result = []
        for v in values:
            variant = self._value_to_variant(v)
            # Always rename if variant text differs from original value
            rename = v if variant != v else None
            result.append((variant, rename))
        return result

    @staticmethod
    def _value_to_variant(v: str) -> str:
        """Convert literal string to Rust enum variant: 'fine-tune' → 'FineTune'."""
        if not v:
            return "Unknown"
        # Handle special numeric prefixes
        clean = v.replace("-", "_").replace(".", "_").replace("/", "_").replace(" ", "_")
        parts = clean.split("_")
        variant = "".join(p.capitalize() for p in parts if p)
        if not variant:
            return "Unknown"
        if variant[0].isdigit():
            variant = "V" + variant
        return variant

    @staticmethod
    def _literal_enum_name(field_name: str, class_name: str) -> str:
        """Generate enum name from field + class context."""
        parts = field_name.split("_")
        camel = "".join(p.capitalize() for p in parts)
        return f"{class_name}{camel}"

    @staticmethod
    def _to_rust_field_name(name: str) -> str:
        """Convert Python field name to valid Rust identifier."""
        # Handle names with special chars (rare)
        clean = name.replace("-", "_").replace(".", "_")
        # Reserved keywords → suffix with _
        if clean in RUST_KEYWORDS:
            return f"{clean}_"
        return clean

    def file_prefix(self, path: Path) -> str:
        """Derive a prefix from filename for dedup."""
        stem = path.stem
        for p in ("response_", "responses_", "chat_completion_", "fine_tuning_"):
            if stem.startswith(p):
                stem = stem[len(p):]
        parts = stem.split("_")
        return "".join(p.capitalize() for p in parts[:2])


# ── Code generator ──

class CodeGen:
    """Generate Rust source code from parsed items."""

    @staticmethod
    def generate_item(item: RustItem) -> str:
        if isinstance(item, RustEnum):
            return CodeGen.generate_enum(item)
        elif isinstance(item, RustStruct):
            return CodeGen.generate_struct(item)
        elif isinstance(item, RustTypeAlias):
            return CodeGen.generate_type_alias(item)
        return ""

    @staticmethod
    def generate_enum(enum: RustEnum) -> str:
        if not enum.name or not enum.name.strip():
            return ""  # Skip enums with empty names
        lines = []
        if enum.doc:
            lines.append(f"/// {enum.doc}")
        lines.append("#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]")
        lines.append('#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]')
        lines.append("#[non_exhaustive]")
        lines.append(f"pub enum {enum.name} {{")
        for variant, rename in enum.variants:
            if rename:
                lines.append(f'    #[serde(rename = "{rename}")]')
            lines.append(f"    {variant},")
        lines.append("}")
        return "\n".join(lines)

    @staticmethod
    def generate_struct(struct: RustStruct) -> str:
        lines = []
        if struct.doc:
            lines.append(f"/// {struct.doc}")
        lines.append("#[derive(Debug, Clone, Serialize, Deserialize)]")
        lines.append('#[cfg_attr(feature = "structured", derive(schemars::JsonSchema))]')
        lines.append(f"pub struct {struct.name} {{")

        for f in struct.fields:
            if f.doc:
                lines.append(f"    /// {f.doc}")

            # serde attributes
            attrs = []
            if f.rename or f.alias:
                rename_val = f.rename or f.alias
                attrs.append(f'rename = "{rename_val}"')
            if f.optional:
                attrs.append('skip_serializing_if = "Option::is_none"')
                attrs.append("default")

            if attrs:
                lines.append(f'    #[serde({", ".join(attrs)})]')
            lines.append(f"    pub {f.rust_name}: {f.rust_type},")

        lines.append("}")
        return "\n".join(lines)

    @staticmethod
    def generate_type_alias(alias: RustTypeAlias) -> str:
        lines = []
        if alias.doc:
            lines.append(f"/// {alias.doc}")
        lines.append(f"pub type {alias.name} = {alias.target};")
        return "\n".join(lines)

    @staticmethod
    def file_header(domain: str, source_hint: str = "", has_manual_siblings: bool = False) -> str:
        source = f" ({source_hint})" if source_hint else ""
        header = (
            f"// AUTO-GENERATED by py2rust — do not edit.\n"
            f"// Re-generate: python3 scripts/py2rust.py sync <python_dir> <rust_dir>\n"
            f"// Domain: {domain}{source}\n"
        )
        # Import types from sibling manual modules so _gen.rs can reference them
        # #![allow] must be at top of file before any items
        if has_manual_siblings:
            header += "#![allow(unused_imports)]\n"
        header += f"\nuse serde::{{Deserialize, Serialize}};\n"
        if has_manual_siblings:
            header += "use super::*;\n"
        return header


# ── Sync engine ──

class SyncEngine:
    """Syncs Python SDK types into the Rust types crate."""

    def __init__(self, python_root: Path, rust_root: Path, dry_run: bool = False):
        self.python_root = python_root
        self.rust_root = rust_root
        self.dry_run = dry_run
        self.parser = Parser()
        self.stats = {"generated": 0, "skipped_manual": 0, "domains": 0}

    def sync_all(self):
        """Walk Python types directory and sync everything."""
        print(f"Syncing {self.python_root} → {self.rust_root}\n")

        # Collect ALL items per domain first, then write once.
        # domain_name → (parser, items_list, seen_set)
        domain_data: dict[str, tuple[Parser, list[RustItem], set[str]]] = {}

        def get_domain(name: str) -> tuple[Parser, list[RustItem], set[str]]:
            if name not in domain_data:
                domain_data[name] = (Parser(), [], set())
            return domain_data[name]

        # Collect all Python files per domain (used for both passes)
        domain_files: dict[str, list[tuple[Path, str]]] = {}  # domain → [(path, prefix)]

        for subdir in sorted(self.python_root.iterdir()):
            if not subdir.is_dir() or subdir.name.startswith("_"):
                continue
            if subdir.name in ("shared_params", "__pycache__"):
                continue
            if subdir.name in SUBDIR_DOMAINS:
                parser, _, _ = get_domain(subdir.name)
                for pyfile in sorted(subdir.glob("*.py")):
                    if pyfile.name.startswith("_") or pyfile.name.endswith("_param.py"):
                        continue
                    prefix = parser.file_prefix(pyfile)
                    domain_files.setdefault(subdir.name, []).append((pyfile, prefix))

        for pyfile in sorted(self.python_root.glob("*.py")):
            if pyfile.name.startswith("_") or pyfile.name.endswith("_param.py"):
                continue
            domain = route_toplevel_file(pyfile.name)
            parser, _, _ = get_domain(domain)
            prefix = parser.file_prefix(pyfile)
            domain_files.setdefault(domain, []).append((pyfile, prefix))

        # Pass 0: prescan domain files to collect type names (enables cross-file refs)
        for domain, files in domain_files.items():
            parser, _, _ = domain_data[domain]
            for pyfile, prefix in files:
                parser.prescan_file(pyfile, prefix=prefix)

        # Pass 1: full parse with all type names known
        for domain, files in domain_files.items():
            parser, items, seen = domain_data[domain]
            for pyfile, prefix in files:
                for item in parser.parse_file(pyfile, prefix=prefix):
                    if item.name not in seen:
                        seen.add(item.name)
                        items.append(item)

        # 3. Write all domains
        for domain in sorted(domain_data):
            _, items, _ = domain_data[domain]
            if items:
                self._write_domain(domain, items)

        # 4. Generate lib.rs
        self._generate_lib_rs()

        # Summary
        print(f"\nDone: {self.stats['generated']} types generated across "
              f"{self.stats['domains']} domains, "
              f"{self.stats['skipped_manual']} skipped (in manual files)")

    def _write_domain(self, domain: str, items: list[RustItem]):
        """Write a domain — creates subdirectory with _gen.rs + mod.rs."""
        rust_dir = self.rust_root / domain
        if not self.dry_run:
            rust_dir.mkdir(parents=True, exist_ok=True)

        # Scan manual types
        manual_types = self._scan_manual_types(rust_dir)

        # Filter out manual types
        gen_items = [it for it in items if it.name not in manual_types]
        skipped = len(items) - len(gen_items)

        if not gen_items and not manual_types:
            return

        # Write _gen.rs
        self._write_gen_file(rust_dir / "_gen.rs", domain, gen_items,
                             has_manual_siblings=bool(manual_types))

        # Write mod.rs
        self._write_mod_rs(rust_dir, domain, manual_types)

        self.stats["domains"] += 1
        self.stats["generated"] += len(gen_items)
        self.stats["skipped_manual"] += skipped

        manual_note = f", {skipped} in manual files" if skipped else ""
        print(f"  {domain}/ — {len(gen_items)} types{manual_note}")

    def _write_gen_file(self, path: Path, domain: str, items: list[RustItem],
                        has_manual_siblings: bool = False):
        """Write a _gen.rs file with all generated items."""
        content = CodeGen.file_header(domain, has_manual_siblings=has_manual_siblings)
        content += "\n"
        generated_code = [CodeGen.generate_item(it) for it in items]
        content += "\n\n".join(c for c in generated_code if c)  # skip empty strings
        content += "\n"

        if self.dry_run:
            print(f"    [dry-run] would write {path} ({len(items)} items)")
        else:
            path.write_text(content)

    def _write_mod_rs(self, rust_dir: Path, domain: str, manual_types: set[str]):
        """Generate mod.rs that re-exports from _gen.rs and manual files."""
        mod_path = rust_dir / "mod.rs"

        # Don't overwrite hand-maintained mod.rs
        if mod_path.exists():
            content = mod_path.read_text()
            if "// MANUAL" in content[:200]:
                return

        lines = [
            f"//! {domain} types — auto-managed by py2rust.\n",
            f"#[allow(clippy::all)]",
            f"mod _gen;",
            f"pub use _gen::*;",
        ]

        # Find manual .rs files in this directory
        for rs_file in sorted(rust_dir.glob("*.rs")):
            name = rs_file.stem
            if name in ("mod", "_gen"):
                continue
            lines.append(f"")
            lines.append(f"pub mod {name};")
            lines.append(f"pub use {name}::*;")

        content = "\n".join(lines) + "\n"

        if self.dry_run:
            print(f"    [dry-run] would write {mod_path}")
        else:
            mod_path.write_text(content)

    def _generate_lib_rs(self):
        """Generate lib.rs with all domain modules."""
        lib_path = self.rust_root / "lib.rs"

        # Don't overwrite hand-maintained lib.rs
        if lib_path.exists():
            content = lib_path.read_text()
            if "// MANUAL" in content[:200]:
                print("  lib.rs — MANUAL, not overwriting")
                return

        lines = [
            "//! Typed OpenAI API models — standalone, zero runtime dependencies beyond serde.",
            "//!",
            "//! Auto-managed by `py2rust.py`. Manual overrides in non-`_gen.rs` files are preserved.",
            "",
        ]

        # All domains are now subdirectories with mod.rs + _gen.rs
        # Each gated behind an optional cargo feature
        for d in sorted(self.rust_root.iterdir()):
            if not d.is_dir() or d.name.startswith("_") or d.name.startswith("."):
                continue
            if (d / "_gen.rs").exists() or (d / "mod.rs").exists():
                # Feature name: underscores → hyphens for cargo convention
                feature = d.name.replace("_", "-")
                lines.append(f'#[cfg(feature = "{feature}")]')
                lines.append(f"pub mod {d.name};")

        lines.append("")

        content = "\n".join(lines) + "\n"

        if self.dry_run:
            print(f"\n  [dry-run] would write {lib_path}")
            print("  Modules:", [l.strip() for l in lines if l.startswith("pub mod")])
        else:
            lib_path.write_text(content)
            print(f"\n  lib.rs — updated")

    def _scan_manual_types(self, rust_dir: Path) -> set[str]:
        """Scan all non-generated .rs files for pub struct/enum names."""
        manual_types: set[str] = set()
        for rs_file in rust_dir.glob("*.rs"):
            if rs_file.stem in ("_gen", "mod"):
                continue
            manual_types |= self._scan_types_in_file(rs_file)
        return manual_types

    @staticmethod
    def _scan_types_in_file(path: Path) -> set[str]:
        """Extract pub struct/enum names from a Rust file."""
        types: set[str] = set()
        if not path.exists():
            return types
        for line in path.read_text().split("\n"):
            line = line.strip()
            if line.startswith("pub struct ") or line.startswith("pub enum "):
                # Extract name: "pub struct Foo {" → "Foo", "pub enum Bar" → "Bar"
                name = line.split("{")[0].split("(")[0].strip().split()[-1]
                if name:
                    types.add(name)
            elif line.startswith("pub type "):
                # "pub type Foo = ..." → "Foo"
                parts = line.split("=")[0].strip().split()
                if len(parts) >= 3:
                    types.add(parts[2])
        return types


# ── CLI ──

def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "sync":
        if len(sys.argv) < 4:
            print("Usage: py2rust.py sync <python_types_dir> <rust_crate_src_dir> [--dry-run]")
            sys.exit(1)
        python_dir = Path(sys.argv[2])
        rust_dir = Path(sys.argv[3])
        dry_run = "--dry-run" in sys.argv
        if not python_dir.is_dir():
            print(f"Error: {python_dir} is not a directory")
            sys.exit(1)
        engine = SyncEngine(python_dir, rust_dir, dry_run=dry_run)
        engine.sync_all()

    elif cmd == "file":
        if len(sys.argv) < 3:
            print("Usage: py2rust.py file <python_file>")
            sys.exit(1)
        path = Path(sys.argv[2])
        parser = Parser()
        items = parser.parse_file(path)
        header = f"// Generated from {path.name}\n\nuse serde::{{Deserialize, Serialize}};\n\n"
        print(header + "\n\n".join(CodeGen.generate_item(it) for it in items))

    else:
        print(f"Unknown command: {cmd}")
        print(__doc__)
        sys.exit(1)


if __name__ == "__main__":
    main()
