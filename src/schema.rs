use crate::config::{FieldSpec, OutputSchema};
use crate::error::{Result, SkillError};
use std::collections::HashMap;

/// Validate extracted values against an output schema.
pub fn validate_schema(
    values: &HashMap<String, Vec<String>>,
    schema: &OutputSchema,
) -> Result<()> {
    for (key, spec) in &schema.fields {
        match values.get(key) {
            Some(vals) => {
                for v in vals {
                    validate_type(key, v, spec)?;
                }
            }
            None => {
                if spec.required {
                    return Err(SkillError::MissingRequired {
                        keys: vec![key.clone()],
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_type(key: &str, value: &str, spec: &FieldSpec) -> Result<()> {
    let ok = match spec.kind.as_str() {
        "string" => true,
        "path" => !value.is_empty(),
        "int" => value.parse::<i64>().is_ok(),
        "float" => value.parse::<f64>().is_ok(),
        "bool" => matches!(value, "true" | "false" | "0" | "1"),
        _ => true,
    };

    if ok {
        Ok(())
    } else {
        Err(SkillError::TypeMismatch {
            key: key.to_string(),
            expected: spec.kind.clone(),
            got: value.to_string(),
        })
    }
}