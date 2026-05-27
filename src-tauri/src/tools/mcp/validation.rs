//! MCP input validation using JSON Schema

use serde_json::Value;

/// Validate tool input against a JSON Schema
pub fn validate_input(schema: &Value, input: &Value) -> Result<(), ValidationError> {
    // Check required fields
    if let Some(required) = schema.get("required").and_then(|r| r.as_array()) {
        let obj = input.as_object().ok_or(ValidationError::NotAnObject)?;
        for field in required {
            let field_name = field.as_str().ok_or(ValidationError::InvalidSchema)?;
            if !obj.contains_key(field_name) {
                return Err(ValidationError::MissingRequired(field_name.to_string()));
            }
        }
    }
    // Check property types
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        let obj = input.as_object().ok_or(ValidationError::NotAnObject)?;
        for (key, prop_schema) in properties {
            if let Some(value) = obj.get(key) {
                if let Some(expected_type) = prop_schema.get("type").and_then(|t| t.as_str()) {
                    validate_type(key, value, expected_type)?;
                }
            }
        }
    }
    Ok(())
}

fn validate_type(field: &str, value: &Value, expected: &str) -> Result<(), ValidationError> {
    let valid = match expected {
        "string" => value.is_string(),
        "number" | "integer" => value.is_number(),
        "boolean" => value.is_boolean(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => true, // unknown types pass through
    };
    if !valid {
        Err(ValidationError::TypeMismatch {
            field: field.to_string(),
            expected: expected.to_string(),
            got: type_name(value),
        })
    } else {
        Ok(())
    }
}

fn type_name(value: &Value) -> String {
    match value {
        Value::String(_) => "string".into(),
        Value::Number(_) => "number".into(),
        Value::Bool(_) => "boolean".into(),
        Value::Array(_) => "array".into(),
        Value::Object(_) => "object".into(),
        Value::Null => "null".into(),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("input is not a JSON object")]
    NotAnObject,
    #[error("invalid JSON Schema")]
    InvalidSchema,
    #[error("missing required field: {0}")]
    MissingRequired(String),
    #[error("field '{field}': expected {expected}, got {got}")]
    TypeMismatch {
        field: String,
        expected: String,
        got: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_required_fields() {
        let schema = serde_json::json!({
            "type": "object",
            "required": ["query"],
            "properties": {
                "query": {"type": "string"}
            }
        });
        assert!(validate_input(&schema, &serde_json::json!({"query": "test"})).is_ok());
        assert!(validate_input(&schema, &serde_json::json!({})).is_err());
    }

    #[test]
    fn validates_types() {
        let schema = serde_json::json!({
            "properties": {
                "count": {"type": "integer"}
            }
        });
        assert!(validate_input(&schema, &serde_json::json!({"count": 42})).is_ok());
        assert!(validate_input(&schema, &serde_json::json!({"count": "not-a-number"})).is_err());
    }

    #[test]
    fn passes_without_schema() {
        let schema = serde_json::json!({});
        assert!(validate_input(&schema, &serde_json::json!({"anything": "goes"})).is_ok());
    }

    #[test]
    fn error_message_contains_field_name() {
        let schema = serde_json::json!({
            "required": ["name"],
            "properties": {
                "name": {"type": "string"}
            }
        });
        let err = validate_input(&schema, &serde_json::json!({})).unwrap_err();
        assert!(err.to_string().contains("name"));
    }

    #[test]
    fn validates_boolean_type() {
        let schema = serde_json::json!({
            "properties": {
                "enabled": {"type": "boolean"}
            }
        });
        assert!(validate_input(&schema, &serde_json::json!({"enabled": true})).is_ok());
        assert!(validate_input(&schema, &serde_json::json!({"enabled": "yes"})).is_err());
    }
}
