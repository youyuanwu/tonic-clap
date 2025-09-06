use bevy_reflect::{TypeInfo, Typed};
use serde_json::Value;

pub fn impl_from_arg_matches<T>(matches: &clap::ArgMatches) -> Result<T, clap::Error>
where
    T: serde::de::DeserializeOwned + Typed,
{
    let type_info = T::type_info();
    let tree = crate::visit::TCFieldType::parse(type_info);

    let mut root_json = serde_json::Map::new();

    tree.visit_nested(&mut |ctx| {
        // Use the prefix, field_name, and field_type to extract values from matches
        assert!(ctx.field_type.is_primitive());
        let arg_name = if ctx.prefix.is_empty() {
            ctx.field_name.clone()
        } else {
            ctx.prefix.join(".") + "." + ctx.field_name
        };

        // Check if argument exists first
        if !matches.contains_id(&arg_name) {
            return;
        }

        // Extract the primitive value
        if let Some(field_value) =
            extract_primitive_value(matches, &arg_name, ctx.field_type.as_primitive())
        {
            // Set the value at the correct nested path
            let mut path_parts = ctx.prefix.to_vec();
            path_parts.push(ctx.field_name.to_string());
            set_nested_value(&mut root_json, &path_parts, field_value);
        }
    });

    // Convert the map to JSON Value
    let json_value = Value::Object(root_json);

    // Use serde to deserialize directly into the target struct
    match serde_json::from_value::<T>(json_value) {
        Ok(instance) => Ok(instance),
        Err(e) => Err(map_serde_error_to_clap(e)),
    }
}

fn map_serde_error_to_clap(e: serde_json::Error) -> clap::Error {
    // If conflicting one of fields args are passed return ArgumentConflict error.
    if e.is_data() && format!("{}", e).contains("expected map with a single key") {
        return clap::Error::raw(
            clap::error::ErrorKind::ArgumentConflict,
            format!("Argument conflict for OneOf fields."),
        );
    }
    clap::Error::raw(
        clap::error::ErrorKind::ValueValidation,
        format!("Failed to deserialize struct from arguments: {}", e),
    )
}

// Helper function to extract primitive value from matches
fn extract_primitive_value(
    matches: &clap::ArgMatches,
    arg_name: &str,
    field_type: &crate::visit::TCFieldTypePrimitive,
) -> Option<Value> {
    use crate::visit::TCFieldTypePrimitive;

    match field_type {
        TCFieldTypePrimitive::String => {
            if let Ok(Some(value)) = matches.try_get_one::<String>(arg_name) {
                Some(Value::String(value.clone()))
            } else {
                None
            }
        }
        TCFieldTypePrimitive::I32 => {
            if let Ok(Some(value)) = matches.try_get_one::<i32>(arg_name) {
                Some(Value::Number(serde_json::Number::from(*value)))
            } else {
                None
            }
        }
        TCFieldTypePrimitive::I64 => {
            if let Ok(Some(value)) = matches.try_get_one::<i64>(arg_name) {
                Some(Value::Number(serde_json::Number::from(*value)))
            } else {
                None
            }
        }
        TCFieldTypePrimitive::F32 => {
            if let Ok(Some(value)) = matches.try_get_one::<f32>(arg_name) {
                serde_json::Number::from_f64(*value as f64).map(Value::Number)
            } else {
                None
            }
        }
        TCFieldTypePrimitive::F64 => {
            if let Ok(Some(value)) = matches.try_get_one::<f64>(arg_name) {
                serde_json::Number::from_f64(*value).map(Value::Number)
            } else {
                None
            }
        }
        TCFieldTypePrimitive::Bool => {
            if let Ok(Some(value)) = matches.try_get_one::<bool>(arg_name) {
                Some(Value::Bool(*value))
            } else if matches.get_flag(arg_name) {
                Some(Value::Bool(true))
            } else {
                None
            }
        }
        TCFieldTypePrimitive::Vec(inner) => {
            if !inner.is_primitive() || (inner.is_primitive() && inner.as_primitive().is_vec()) {
                // skip nested vec.
                None
            } else {
                // Handle vectors - for now assume Vec<String>
                if let Ok(Some(values)) = matches.try_get_many::<String>(arg_name) {
                    let vec_values: Vec<String> = values.cloned().collect();
                    let json_array: Vec<Value> =
                        vec_values.into_iter().map(Value::String).collect();
                    Some(Value::Array(json_array))
                } else {
                    None
                }
            }
        }
        _ => {
            // Default to string for other types
            if let Ok(Some(value)) = matches.try_get_one::<String>(arg_name) {
                Some(Value::String(value.clone()))
            } else {
                None
            }
        }
    }
}

// Helper function to set a value at a nested path in JSON
fn set_nested_value(root: &mut serde_json::Map<String, Value>, path: &[String], value: Value) {
    if path.is_empty() {
        // This shouldn't happen, but handle gracefully
        return;
    }

    if path.len() == 1 {
        // Base case: set the final value
        root.insert(path[0].clone(), value);
    } else {
        // Recursive case: navigate deeper
        let key = &path[0];
        let remaining_path = &path[1..];

        // Get or create the nested object
        let entry = root
            .entry(key.clone())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));

        // Ensure it's an object and recurse
        if let Value::Object(nested_map) = entry {
            set_nested_value(nested_map, remaining_path, value);
        } else {
            // Replace non-object with object and recurse
            *entry = Value::Object(serde_json::Map::new());
            if let Value::Object(nested_map) = entry {
                set_nested_value(nested_map, remaining_path, value);
            }
        }
    }
}

pub fn impl_augment_args(mut cmd: clap::Command, type_info: &TypeInfo) -> clap::Command {
    let tree = crate::visit::TCFieldType::parse(type_info);
    let mut args = Vec::new();
    tree.visit_nested(&mut |ctx| {
        assert!(ctx.field_type.is_primitive());
        let arg_name = if ctx.prefix.is_empty() {
            ctx.field_name.clone()
        } else {
            ctx.prefix.join(".") + "." + ctx.field_name
        };
        let primitive_type = ctx.field_type.as_primitive();
        if primitive_type.is_vec() && !primitive_type.is_primitive_vec() {
            // We only support primitive vec for now.
            return;
        }
        let help_text = format!("Arg: {}", primitive_type.display_primitive_type());

        let (value_parser, action) = primitive_type.get_clap_value_parse();
        let arg = clap::Arg::new(&arg_name)
            .long(&arg_name)
            .value_name(ctx.field_name.to_uppercase())
            .help(&help_text)
            .required(false) // TODO: support required properly. Currently many protos does not indicate if field is required.
            .action(action)
            .value_parser(value_parser.clone());
        args.push(arg);
    });
    for arg in args {
        cmd = cmd.arg(arg);
    }
    cmd
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn json_error_test(){
//         let mut root_json = serde_json::Map::new();
//         root_json.insert("field1".to_string(), serde_json::Value::String("value1".to_string()));
//         root_json.insert("field1".to_string(), serde_json::Value::String("value2".to_string()));
//     }
// }
