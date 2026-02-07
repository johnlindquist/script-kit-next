fn field_to_json_schema(field: &FieldDef) -> serde_json::Value {
    let mut schema = serde_json::Map::new();

    // JSON Schema: "any" is not a valid type, so represent it as a union.
    match field.field_type {
        FieldType::Any => {
            schema.insert(
                "type".to_string(),
                serde_json::json!(["string", "number", "boolean", "object", "array", "null"]),
            );
        }
        _ => {
            let type_str = match field.field_type {
                FieldType::String => "string",
                FieldType::Number => "number",
                FieldType::Boolean => "boolean",
                FieldType::Array => "array",
                FieldType::Object => "object",
                FieldType::Any => unreachable!("handled above"),
            };
            schema.insert(
                "type".to_string(),
                serde_json::Value::String(type_str.to_string()),
            );
        }
    }

    if let Some(desc) = &field.description {
        schema.insert(
            "description".to_string(),
            serde_json::Value::String(desc.clone()),
        );
    }

    if let Some(default) = &field.default {
        schema.insert("default".to_string(), default.clone());
    }

    if let Some(enum_vals) = &field.enum_values {
        let vals: Vec<serde_json::Value> = enum_vals
            .iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect();
        schema.insert("enum".to_string(), serde_json::Value::Array(vals));
    }

    // min/max: apply correctly depending on field type
    if let Some(min) = field.min {
        match field.field_type {
            FieldType::Number => {
                schema.insert("minimum".to_string(), serde_json::json!(min));
            }
            FieldType::String => {
                schema.insert("minLength".to_string(), serde_json::json!(min as u64));
            }
            FieldType::Array => {
                schema.insert("minItems".to_string(), serde_json::json!(min as u64));
            }
            FieldType::Object => {
                schema.insert("minProperties".to_string(), serde_json::json!(min as u64));
            }
            _ => {}
        }
    }

    if let Some(max) = field.max {
        match field.field_type {
            FieldType::Number => {
                schema.insert("maximum".to_string(), serde_json::json!(max));
            }
            FieldType::String => {
                schema.insert("maxLength".to_string(), serde_json::json!(max as u64));
            }
            FieldType::Array => {
                schema.insert("maxItems".to_string(), serde_json::json!(max as u64));
            }
            FieldType::Object => {
                schema.insert("maxProperties".to_string(), serde_json::json!(max as u64));
            }
            _ => {}
        }
    }

    if let Some(pattern) = &field.pattern {
        schema.insert(
            "pattern".to_string(),
            serde_json::Value::String(pattern.clone()),
        );
    }

    // items: support both string shorthand and full schema object
    if let Some(items) = &field.items {
        let items_schema = match items {
            ItemsDef::Type(t) => serde_json::json!({ "type": t }),
            ItemsDef::Schema(def) => field_to_json_schema(def),
        };
        schema.insert("items".to_string(), items_schema);
    }

    // properties: include nested required keys (useful and safe)
    if let Some(props) = &field.properties {
        let mut prop_schemas = serde_json::Map::new();
        let mut required_props: Vec<serde_json::Value> = Vec::new();

        for (name, prop_field) in props {
            prop_schemas.insert(name.clone(), field_to_json_schema(prop_field));
            if prop_field.required {
                required_props.push(serde_json::Value::String(name.clone()));
            }
        }

        schema.insert(
            "properties".to_string(),
            serde_json::Value::Object(prop_schemas),
        );

        if !required_props.is_empty() {
            schema.insert(
                "required".to_string(),
                serde_json::Value::Array(required_props),
            );
        }
    }

    if let Some(example) = &field.example {
        schema.insert("example".to_string(), example.clone());
    }

    serde_json::Value::Object(schema)
}
