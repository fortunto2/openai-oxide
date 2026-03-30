#!/usr/bin/env python3
"""Generate TypeScript type definitions from openai-types Rust source.

Reads all pub struct/enum definitions and converts to .d.ts interfaces.
Output: openai-oxide-node/types.d.ts

Usage: python3 scripts/gen_ts_types.py
"""

import re
from pathlib import Path

RUST_TO_TS = {
    "String": "string",
    "str": "string",
    "bool": "boolean",
    "i8": "number", "i16": "number", "i32": "number", "i64": "number",
    "u8": "number", "u16": "number", "u32": "number", "u64": "number",
    "f32": "number", "f64": "number",
    "serde_json::Value": "any",
    "serde_json::Map<String, serde_json::Value>": "Record<string, any>",
    "Vec<u8>": "Uint8Array",
}

def rust_type_to_ts(rust_type: str) -> str:
    rust_type = rust_type.strip().rstrip(",")

    # Option<T> -> T | undefined
    m = re.match(r"Option<(.+)>", rust_type)
    if m:
        inner = rust_type_to_ts(m.group(1))
        return f"{inner} | undefined"

    # Vec<T> -> T[]
    m = re.match(r"Vec<(.+)>", rust_type)
    if m:
        inner = rust_type_to_ts(m.group(1))
        return f"{inner}[]"

    # HashMap<K, V> -> Record<K, V>
    m = re.match(r"(?:std::collections::)?HashMap<(.+),\s*(.+)>", rust_type)
    if m:
        k = rust_type_to_ts(m.group(1))
        v = rust_type_to_ts(m.group(2))
        return f"Record<{k}, {v}>"

    # Direct mapping
    if rust_type in RUST_TO_TS:
        return RUST_TO_TS[rust_type]

    # Known types stay as-is (PascalCase = interface reference)
    if rust_type and rust_type[0].isupper():
        return rust_type

    return "any"


def parse_struct(name: str, body: str) -> list[tuple[str, str, bool]]:
    """Parse struct fields -> [(ts_name, ts_type, optional)]"""
    fields = []
    for line in body.split("\n"):
        line = line.strip()
        if not line.startswith("pub "):
            continue
        # pub field_name: Type,
        m = re.match(r"pub (\w+): (.+)", line)
        if not m:
            continue
        field_name = m.group(1)
        rust_type = m.group(2).rstrip(",").strip()

        optional = rust_type.startswith("Option<")
        ts_type = rust_type_to_ts(rust_type)

        # serde rename
        # Look for #[serde(rename = "...")] above
        ts_name = to_camel_case(field_name) if field_name != field_name.lower() else field_name
        # Keep snake_case for JSON compat (OpenAI API uses snake_case)
        ts_name = field_name

        fields.append((ts_name, ts_type, optional))
    return fields


def parse_enum_variants(body: str) -> list[str]:
    """Parse enum variants -> list of string literal values"""
    values = []
    for line in body.split("\n"):
        line = line.strip()
        # #[serde(rename = "value")]
        m = re.search(r'#\[serde\(rename\s*=\s*"([^"]+)"\)', line)
        if m:
            values.append(m.group(1))
            continue
        # VariantName, (without rename = use snake_case)
        m = re.match(r"^(\w+),?$", line)
        if m and m.group(1) not in ("Other",):
            variant = m.group(1)
            # Convert PascalCase to snake_case for JSON
            snake = re.sub(r"([A-Z])", r"_\1", variant).lower().strip("_")
            values.append(snake)
    return values


def to_camel_case(s: str) -> str:
    parts = s.split("_")
    return parts[0] + "".join(p.capitalize() for p in parts[1:])


def generate_types(rust_root: Path) -> str:
    lines = [
        "// Auto-generated TypeScript types from openai-types Rust crate.",
        "// Do not edit manually. Regenerate: python3 scripts/gen_ts_types.py",
        "//",
        f"// Generated from {sum(1 for _ in rust_root.rglob('*.rs'))} Rust source files.",
        "",
        "export namespace OpenAI {",
    ]

    # Process each domain
    for domain_dir in sorted(rust_root.iterdir()):
        if not domain_dir.is_dir() or domain_dir.name.startswith("_"):
            continue

        domain_name = domain_dir.name
        structs = []
        enums = []

        for rs_file in sorted(domain_dir.glob("*.rs")):
            if rs_file.stem == "mod":
                continue
            text = rs_file.read_text()

            # Extract structs
            for m in re.finditer(
                r"pub struct (\w+)\s*\{([^}]*)\}", text, re.DOTALL
            ):
                name = m.group(1)
                fields = parse_struct(name, m.group(2))
                if fields:
                    structs.append((name, fields))

            # Extract enums with string variants
            for m in re.finditer(
                r"pub enum (\w+)\s*\{([^}]*)\}", text, re.DOTALL
            ):
                name = m.group(1)
                variants = parse_enum_variants(m.group(2))
                if variants:
                    enums.append((name, variants))

        if not structs and not enums:
            continue

        lines.append(f"  export namespace {domain_name} {{")

        # Enums as union types
        for name, variants in enums:
            # Deduplicate variants (serde rename + snake_case can produce dupes)
            seen = []
            for v in variants:
                if v not in seen:
                    seen.append(v)
            variants = seen
            quoted = " | ".join(f'"{v}"' for v in variants)
            lines.append(f"    export type {name} = {quoted};")

        if enums and structs:
            lines.append("")

        # Structs as interfaces
        for name, fields in structs:
            lines.append(f"    export interface {name} {{")
            for ts_name, ts_type, optional in fields:
                opt = "?" if optional else ""
                lines.append(f"      {ts_name}{opt}: {ts_type};")
            lines.append("    }")
            lines.append("")

        lines.append("  }")
        lines.append("")

    lines.append("}")
    lines.append("")

    # Export convenience types for main API
    lines.extend([
        "// Convenience re-exports for common types",
        "export type ChatCompletionRequest = OpenAI.chat.ChatCompletionRequest;",
        "export type ChatCompletionResponse = OpenAI.chat.ChatCompletionResponse;",
        "export type ResponseCreateRequest = OpenAI.responses.ResponseCreateRequest;",
        "export type Response = OpenAI.responses.Response;",
        "",
    ])

    return "\n".join(lines)


def main():
    rust_root = Path("openai-types/src")
    output = Path("openai-oxide-node/types.d.ts")

    ts = generate_types(rust_root)
    output.write_text(ts)

    # Count generated types
    interfaces = ts.count("export interface")
    type_aliases = ts.count("export type")
    print(f"Generated {output}: {interfaces} interfaces, {type_aliases} type aliases")


if __name__ == "__main__":
    main()
