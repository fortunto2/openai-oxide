//! Utilities for normalizing JSON schemas for OpenAI API compatibility.
//!
//! OpenAI's function calling has stricter schema requirements than some other providers.
//! This module provides helpers to transform schemas (e.g., from Anthropic's format)
//! into OpenAI-compatible form.

use serde_json::Value;

/// Normalizes a tool schema for OpenAI function calling compatibility.
///
/// Applies the following transformations:
/// - Object schemas MUST have `properties` — adds empty `{}` if missing.
/// - Removes `additionalProperties: true` (OpenAI forbids it; keeps `false` if present).
/// - Union types like `{"type": ["string", "boolean"]}` — removes the `type` constraint entirely.
/// - Recursively normalizes nested object schemas within `properties`.
/// - Recursively normalizes `items` in array schemas.
///
/// Returns `None` if the input is not a JSON object (fundamentally invalid schema).
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use openai_oxide::schema::normalize_tool_schema;
///
/// let schema = json!({"type": "object", "additionalProperties": true});
/// let normalized = normalize_tool_schema(&schema).unwrap();
/// assert_eq!(normalized, json!({"type": "object", "properties": {}}));
/// ```
pub fn normalize_tool_schema(schema: &Value) -> Option<Value> {
    let obj = schema.as_object()?;
    let mut result = obj.clone();

    // Handle union types: {"type": ["string", "boolean"]} → remove type
    if result.get("type").is_some_and(|t| t.is_array()) {
        result.remove("type");
        return Some(Value::Object(result));
    }

    // Handle additionalProperties — remove `true`, keep `false`
    if result
        .get("additionalProperties")
        .is_some_and(|ap| ap == &Value::Bool(true))
    {
        result.remove("additionalProperties");
    }

    // For object schemas, ensure `properties` exists and recurse into them
    let is_object = result
        .get("type")
        .and_then(|t| t.as_str())
        .map(|s| s == "object")
        .unwrap_or(false);

    if is_object {
        if !result.contains_key("properties") {
            result.insert("properties".to_string(), Value::Object(Default::default()));
        }

        // Recursively normalize each property schema
        if let Some(Value::Object(props)) = result.get("properties").cloned() {
            let mut normalized_props = serde_json::Map::new();
            for (key, prop_schema) in props {
                if let Some(normalized) = normalize_tool_schema(&prop_schema) {
                    normalized_props.insert(key, normalized);
                } else {
                    // Non-object property values pass through unchanged
                    normalized_props.insert(key, prop_schema);
                }
            }
            result.insert("properties".to_string(), Value::Object(normalized_props));
        }
    }

    // For array schemas, recursively normalize `items`
    let is_array = result
        .get("type")
        .and_then(|t| t.as_str())
        .map(|s| s == "array")
        .unwrap_or(false);

    if is_array
        && let Some(items) = result.get("items").cloned()
        && let Some(normalized_items) = normalize_tool_schema(&items)
    {
        result.insert("items".to_string(), normalized_items);
    }

    Some(Value::Object(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalizes_schema_missing_properties() {
        let schema = json!({"type": "object", "additionalProperties": true});
        let result = normalize_tool_schema(&schema).unwrap();
        assert_eq!(result, json!({"type": "object", "properties": {}}));
    }

    #[test]
    fn normalizes_union_type() {
        let schema = json!({
            "type": "object",
            "properties": {
                "value": {
                    "type": ["string", "boolean", "number"]
                }
            }
        });
        let result = normalize_tool_schema(&schema).unwrap();
        // The union type property should have `type` removed
        let value_prop = &result["properties"]["value"];
        assert!(value_prop.get("type").is_none());
    }

    #[test]
    fn preserves_valid_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            },
            "required": ["name", "age"]
        });
        let result = normalize_tool_schema(&schema).unwrap();
        assert_eq!(result, schema);
    }

    #[test]
    fn normalizes_nested_objects() {
        let schema = json!({
            "type": "object",
            "properties": {
                "address": {
                    "type": "object",
                    "additionalProperties": true
                }
            }
        });
        let result = normalize_tool_schema(&schema).unwrap();
        let address = &result["properties"]["address"];
        // Nested object should have properties added and additionalProperties removed
        assert_eq!(*address, json!({"type": "object", "properties": {}}));
    }

    #[test]
    fn returns_none_for_non_object() {
        assert!(normalize_tool_schema(&json!("string")).is_none());
        assert!(normalize_tool_schema(&json!(42)).is_none());
        assert!(normalize_tool_schema(&json!(null)).is_none());
        assert!(normalize_tool_schema(&json!(true)).is_none());
    }

    #[test]
    fn preserves_additional_properties_false() {
        let schema = json!({
            "type": "object",
            "properties": {"x": {"type": "string"}},
            "additionalProperties": false
        });
        let result = normalize_tool_schema(&schema).unwrap();
        assert_eq!(result["additionalProperties"], json!(false));
    }

    #[test]
    fn normalizes_array_items() {
        let schema = json!({
            "type": "object",
            "properties": {
                "tags": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "additionalProperties": true
                    }
                }
            }
        });
        let result = normalize_tool_schema(&schema).unwrap();
        let items = &result["properties"]["tags"]["items"];
        assert_eq!(*items, json!({"type": "object", "properties": {}}));
    }
}
