use bevy_reflect::{TypeInfo, Typed};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;

// 🎯 TYPE INFORMATION STORE: Store field types during reflection for intelligent parsing
#[allow(dead_code)]
#[derive(Debug, Clone)]
enum StoredFieldType {
    String,
    I32,
    I64,
    F32,
    F64,
    Bool,
    VecString,
    OptionString,
    OptionI32,
    OptionCustom(String), // For Option<CustomStruct>
    Custom(String),       // For custom structs
}

// 🎯 TYPE INFORMATION STORE: Thread-local storage for field types
thread_local! {
    static TYPE_INFO_STORE: RefCell<HashMap<String, StoredFieldType>> = RefCell::new(HashMap::new());
}

fn store_field_type_for_struct(struct_name: &str, arg_name: &str, field_type: StoredFieldType) {
    let key = format!("{}::{}", struct_name, arg_name);
    TYPE_INFO_STORE.with(|store| {
        store.borrow_mut().insert(key, field_type);
    });
}

fn get_field_type_for_struct(struct_name: &str, arg_name: &str) -> Option<StoredFieldType> {
    let key = format!("{}::{}", struct_name, arg_name);
    TYPE_INFO_STORE.with(|store| store.borrow().get(&key).cloned())
}

// Backward compatibility functions - try to infer struct name from context
fn store_field_type(arg_name: &str, field_type: StoredFieldType) {
    // Try to get current struct name from some context, or use a default
    let struct_name = get_current_struct_name().unwrap_or_else(|| "Unknown".to_string());
    store_field_type_for_struct(&struct_name, arg_name, field_type);
}

fn get_field_type(arg_name: &str) -> Option<StoredFieldType> {
    // Try with current struct context first if available
    let struct_name = get_current_struct_name().expect("current struct not set");
    // Direct field lookup for current struct
    if let Some(field_type) = get_field_type_for_struct(&struct_name, arg_name) {
        return Some(field_type);
    }

    // Handle nested fields like "field1.fname" within current struct context
    if arg_name.contains('.') {
        let parts: Vec<&str> = arg_name.split('.').collect();
        if parts.len() == 2 {
            let _prefix = parts[0];
            let field_name = parts[1];

            // Look for the nested field in current struct
            if let Some(field_type) = get_field_type_for_struct(&struct_name, field_name) {
                return Some(field_type);
            }

            // Try with the full nested path in current struct
            if let Some(field_type) = get_field_type_for_struct(&struct_name, arg_name) {
                return Some(field_type);
            }
        }
    }
    // TODO: this fallback is not robust. May still need to do extract along with reflect.
    // TYPE_INFO_STORE
    //     .with_borrow(|store| panic!("no type info stored for struct {struct_name} arg {arg_name}. {:?}", store));
    // panic!("not found");
    // Fallback: Search through all stored types to find a match
    TYPE_INFO_STORE.with(|store| {
        let store = store.borrow();

        // For nested fields, try both the full arg_name and just the field part
        if arg_name.contains('.') {
            let parts: Vec<&str> = arg_name.split('.').collect();
            if parts.len() == 2 {
                let field_name = parts[1];

                // Try exact match first
                for (key, value) in store.iter() {
                    if key.ends_with(&format!("::{}", arg_name)) {
                        return Some(value.clone());
                    }
                }

                // Try field name match
                for (key, value) in store.iter() {
                    if key.ends_with(&format!("::{}", field_name)) {
                        return Some(value.clone());
                    }
                }
            }
        }

        // Direct field lookup
        for (key, value) in store.iter() {
            if key.ends_with(&format!("::{}", arg_name)) {
                return Some(value.clone());
            }
        }

        None
    })
}

// 🎯 NEW SERDE APPROACH: Extract arguments into JSON and use serde to construct structs
// This eliminates ALL downcasting and hardcoded type matching!
fn extract_args_to_json_map(matches: &clap::ArgMatches) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    let mut nested_maps: HashMap<String, HashMap<String, Value>> = HashMap::new();

    // Get the current struct name to filter arguments
    let current_struct_name = get_current_struct_name().expect("current struct not set");

    // Get all argument IDs from the matches
    for arg_id in matches.ids() {
        let arg_name = arg_id.as_str();

        // When we have a current struct name, only process arguments that belong to it
        // Check if this argument belongs to the current struct
        if get_field_type_for_struct(&current_struct_name, arg_name).is_some() {
            // This is a direct field of the current struct
            extract_argument_value(matches, arg_name, arg_name, &mut map);
            continue;
        }

        // Check if this is a nested argument (contains '.') that belongs to current struct
        if let Some(dot_pos) = arg_name.find('.') {
            let nested_field_name = &arg_name[..dot_pos];
            let inner_field_name = &arg_name[dot_pos + 1..];

            // Check if the nested field belongs to current struct (e.g., field1 belongs to HelloRequest2)
            if get_field_type_for_struct(&current_struct_name, nested_field_name).is_some() {
                // Create nested structure
                if !nested_maps.contains_key(nested_field_name) {
                    nested_maps.insert(nested_field_name.to_string(), HashMap::new());
                }

                // Extract the value and add to nested map
                let nested_map = nested_maps.get_mut(nested_field_name).unwrap();
                extract_argument_value(matches, arg_name, inner_field_name, nested_map);
                continue;
            }
        }
        panic!("arg_name not found {arg_name}");
    }

    // Convert nested maps to JSON objects and add to main map
    for (nested_name, nested_map) in nested_maps {
        if !nested_map.is_empty() {
            let nested_json: serde_json::Map<String, Value> = nested_map.into_iter().collect();
            map.insert(nested_name, Value::Object(nested_json));
        }
    }

    map
}

// Helper function to extract argument values using stored type information
fn extract_argument_value(
    matches: &clap::ArgMatches,
    arg_name: &str,
    field_name: &str,
    map: &mut HashMap<String, Value>,
) {
    // Check if argument exists first
    if !matches.contains_id(arg_name) {
        return;
    }

    // 🎯 Use stored type information for intelligent extraction
    if let Some(stored_type) = get_field_type(arg_name) {
        match stored_type {
            StoredFieldType::String => {
                if let Ok(Some(value)) = matches.try_get_one::<String>(arg_name) {
                    map.insert(field_name.to_string(), Value::String(value.clone()));
                }
            }
            StoredFieldType::I32 => {
                if let Ok(Some(value)) = matches.try_get_one::<i32>(arg_name) {
                    map.insert(
                        field_name.to_string(),
                        Value::Number(serde_json::Number::from(*value)),
                    );
                }
            }
            StoredFieldType::I64 => {
                if let Ok(Some(value)) = matches.try_get_one::<i64>(arg_name) {
                    map.insert(
                        field_name.to_string(),
                        Value::Number(serde_json::Number::from(*value)),
                    );
                }
            }
            StoredFieldType::F32 => {
                if let Ok(Some(value)) = matches.try_get_one::<f32>(arg_name)
                    && let Some(num) = serde_json::Number::from_f64(*value as f64)
                {
                    map.insert(field_name.to_string(), Value::Number(num));
                }
            }
            StoredFieldType::F64 => {
                if let Ok(Some(value)) = matches.try_get_one::<f64>(arg_name)
                    && let Some(num) = serde_json::Number::from_f64(*value)
                {
                    map.insert(field_name.to_string(), Value::Number(num));
                }
            }
            StoredFieldType::Bool => {
                if let Ok(Some(value)) = matches.try_get_one::<bool>(arg_name) {
                    map.insert(field_name.to_string(), Value::Bool(*value));
                } else if matches.get_flag(arg_name) {
                    map.insert(field_name.to_string(), Value::Bool(true));
                }
            }
            StoredFieldType::VecString => {
                if let Ok(Some(values)) = matches.try_get_many::<String>(arg_name) {
                    let vec_values: Vec<String> = values.cloned().collect();
                    let json_array: Vec<Value> =
                        vec_values.into_iter().map(Value::String).collect();
                    map.insert(field_name.to_string(), Value::Array(json_array));
                }
            }
            StoredFieldType::OptionString => {
                if let Ok(Some(value)) = matches.try_get_one::<String>(arg_name) {
                    map.insert(field_name.to_string(), Value::String(value.clone()));
                }
            }
            StoredFieldType::OptionI32 => {
                if let Ok(Some(value)) = matches.try_get_one::<i32>(arg_name) {
                    map.insert(
                        field_name.to_string(),
                        Value::Number(serde_json::Number::from(*value)),
                    );
                }
            }
            StoredFieldType::OptionCustom(_) | StoredFieldType::Custom(_) => {
                // For custom types, try string as fallback
                if let Ok(Some(value)) = matches.try_get_one::<String>(arg_name) {
                    map.insert(field_name.to_string(), Value::String(value.clone()));
                }
            }
        }
    } else {
        TYPE_INFO_STORE
            .with_borrow(|store| panic!("no type info stored for arg {arg_name}. {:?}", store));
    }
}

// 🎯 SERDE-BASED STRUCT POPULATION: No downcasting, no hardcoded types!
pub fn impl_from_arg_matches<T>(matches: &clap::ArgMatches) -> Result<T, clap::Error>
where
    T: serde::de::DeserializeOwned + Typed,
{
    // Set the struct name for type-aware extraction
    let struct_name = T::type_info().type_path_table().path();
    set_current_struct_name(struct_name);

    // Extract all arguments into a JSON-compatible map
    let args_map = extract_args_to_json_map(matches);

    // Clear struct name after extraction
    clear_current_struct_name();

    // Convert the map to JSON Value
    let json_value = Value::Object(args_map.into_iter().collect());

    // Use serde to deserialize directly into the target struct
    match serde_json::from_value::<T>(json_value) {
        Ok(instance) => Ok(instance),
        Err(e) => Err(clap::Error::raw(
            clap::error::ErrorKind::ValueValidation,
            format!("Failed to deserialize struct from arguments: {}", e),
        )),
    }
}

pub fn impl_augment_args(mut cmd: clap::Command, type_info: &TypeInfo) -> clap::Command {
    let prefix = get_prefix();

    if let TypeInfo::Struct(struct_info) = type_info {
        // Extract and store the struct name for type information storage
        let struct_name = struct_info.type_path_table().path();
        set_current_struct_name(struct_name);

        for field in struct_info.iter() {
            let field_name = field.name();
            let field_type_path = field.type_path();

            // Generate argument name with optional prefix
            let arg_name = if let Some(p) = &prefix {
                format!("{}.{}", p, field_name)
            } else {
                field_name.to_string()
            };
            let arg_name_static: &'static str = Box::leak(arg_name.clone().into_boxed_str());

            // Generate help text
            let help_text = if let Some(p) = &prefix {
                format!("{} {} (reflected with prefix)", p, field_name)
            } else {
                format!("{} (reflected without prefix)", field_name)
            };

            // Generate value name (uppercase field name)
            let value_name = field_name.to_uppercase();
            let value_name_static: &'static str = Box::leak(value_name.into_boxed_str());

            // Create argument based on field type using reflection
            if field_type_path.contains("String") && !field_type_path.contains("Vec") {
                // Store type information
                store_field_type(&arg_name, StoredFieldType::String);

                cmd = cmd.arg(
                    clap::Arg::new(arg_name_static)
                        .long(arg_name_static)
                        .value_name(value_name_static)
                        .help(&help_text)
                        .required(false)
                        .action(clap::ArgAction::Set),
                );
            } else if field_type_path.contains("Option") {
                // Check if the inner type might be a struct with Args trait
                if !field_type_path.contains("String")
                    && !field_type_path.contains("i32")
                    && !field_type_path.contains("i64")
                    && !field_type_path.contains("bool")
                    && !field_type_path.contains("f32")
                    && !field_type_path.contains("f64")
                    && !field_type_path.contains("Vec")
                {
                    // This is an Option<CustomStruct> - recursively handle nested struct

                    // Store type information for the Option field itself
                    store_field_type(
                        field_name,
                        StoredFieldType::OptionCustom(field_type_path.to_string()),
                    );

                    // Save the current struct name before switching context
                    let current_struct = get_current_struct_name();

                    set_prefix(field_name); // Use field name as prefix

                    // Try to get the inner type's TypeInfo using reflection
                    let inner_type_info = field.type_info().unwrap();
                    // For Option<T>, we need to extract T's type info
                    // Option is represented as an enum with Some(T) and None variants
                    if let bevy_reflect::TypeInfo::Enum(enum_info) = inner_type_info {
                        // Check if this is an Option type by looking for Some/None variants
                        if enum_info.variant("Some").is_some()
                            && enum_info.variant("None").is_some()
                        {
                            // Get the Some variant which contains the inner type
                            let some_variant = enum_info.variant("Some").unwrap_or_else(|| {
                                panic!("Could not find Some variant for: {}", field_type_path)
                            });
                            // Option's Some variant is a tuple variant with one field containing T
                            let tuple_variant =
                                some_variant.as_tuple_variant().unwrap_or_else(|_| {
                                    panic!(
                                        "Some variant is not a tuple variant for: {}",
                                        field_type_path
                                    )
                                });
                            let inner_field = tuple_variant.field_at(0).unwrap_or_else(|| {
                                panic!(
                                    "Could not access inner field at index 0 for: {}",
                                    field_type_path
                                )
                            });
                            // Recursively call with the inner type's info
                            let inner_type_info = inner_field.type_info().unwrap_or_else(|| {
                                panic!(
                                    "Could not get type info for inner field: {}",
                                    field_type_path
                                )
                            });
                            cmd = impl_augment_args(cmd, inner_type_info);
                        } else {
                            panic!(
                                "Field is not an Option type (no Some/None variants): {}",
                                field_type_path
                            );
                        }
                    } else {
                        panic!("Field type info is not an enum: {}", field_type_path);
                    }

                    clear_prefix();

                    // Restore the previous struct name context
                    if let Some(struct_name) = current_struct {
                        set_current_struct_name(&struct_name);
                    } else {
                        clear_current_struct_name();
                    }
                } else {
                    // Simple optional primitive type - determine which one
                    if field_type_path.contains("String") {
                        store_field_type(&arg_name, StoredFieldType::OptionString);
                    } else if field_type_path.contains("i32") {
                        store_field_type(&arg_name, StoredFieldType::OptionI32);
                    } else {
                        store_field_type(&arg_name, StoredFieldType::String); // Default fallback
                    }

                    cmd = cmd.arg(
                        clap::Arg::new(arg_name_static)
                            .long(arg_name_static)
                            .value_name(value_name_static)
                            .help(&help_text)
                            .required(false)
                            .action(clap::ArgAction::Set),
                    );
                }
            } else if field_type_path.contains("Vec") && field_type_path.contains("String") {
                // Store type information for Vec<String>
                store_field_type(&arg_name, StoredFieldType::VecString);

                cmd = cmd.arg(
                    clap::Arg::new(arg_name_static)
                        .long(arg_name_static)
                        .value_name(value_name_static)
                        .help(&help_text)
                        .required(false)
                        .action(clap::ArgAction::Append),
                );
            } else if field_type_path.contains("i32") {
                // Store type information for i32
                store_field_type(&arg_name, StoredFieldType::I32);

                // i32 field - treat as plain integer
                cmd = cmd.arg(
                    clap::Arg::new(arg_name_static)
                        .long(arg_name_static)
                        .value_name(value_name_static)
                        .help(format!("{} (i32, reflected)", help_text))
                        .required(false)
                        .action(clap::ArgAction::Set)
                        .value_parser(clap::value_parser!(i32)),
                );
            } else {
                // Generic fallback for other types
                if field_type_path.contains("Vec") {
                    store_field_type(&arg_name, StoredFieldType::VecString); // Default to Vec<String>

                    cmd = cmd.arg(
                        clap::Arg::new(arg_name_static)
                            .long(arg_name_static)
                            .value_name(value_name_static)
                            .help(format!("{} (vector, reflected)", help_text))
                            .required(false)
                            .action(clap::ArgAction::Append),
                    );
                } else {
                    store_field_type(&arg_name, StoredFieldType::String); // Default to String

                    cmd = cmd.arg(
                        clap::Arg::new(arg_name_static)
                            .long(arg_name_static)
                            .value_name(value_name_static)
                            .help(format!("{} (reflected)", help_text))
                            .required(false)
                            .action(clap::ArgAction::Set),
                    );
                }
            }
            // Add more type handling as needed - all done via reflection!
        }

        // Clear the current struct name after processing
        clear_current_struct_name();
    }

    cmd
}

thread_local! {
    static LAST_PREFIX: RefCell<Option<String>> = const { RefCell::new(None) };
    static CURRENT_STRUCT: RefCell<Option<String>> = const { RefCell::new(None) };
}

fn set_current_struct_name(struct_name: &str) {
    CURRENT_STRUCT.with(|s| {
        *s.borrow_mut() = Some(struct_name.to_string());
    });
}

fn get_current_struct_name() -> Option<String> {
    CURRENT_STRUCT.with(|s| s.borrow().clone())
}

fn clear_current_struct_name() {
    CURRENT_STRUCT.with(|s| {
        *s.borrow_mut() = None;
    });
}

fn set_prefix(prefix: &str) {
    LAST_PREFIX.with(|p| {
        *p.borrow_mut() = Some(prefix.to_string());
    });
}

fn get_prefix() -> Option<String> {
    LAST_PREFIX.with(|p| p.borrow().clone())
}

fn clear_prefix() {
    LAST_PREFIX.with(|p| {
        *p.borrow_mut() = None;
    });
}

// Example demonstrating pure reflection-based field setting without hardcoded field names
#[cfg(test)]
mod tests {
    use super::*;
    use clap::{Args, Command, FromArgMatches};

    #[derive(
        serde::Serialize,
        serde::Deserialize,
        Clone,
        PartialEq,
        Eq,
        Hash,
        ::prost::Message,
        bevy_reflect::Reflect,
    )]
    #[serde(default)]
    pub struct HelloRequest2 {
        #[prost(string, tag = "1")]
        pub name: ::prost::alloc::string::String,
        #[prost(message, optional, tag = "2")]
        pub field1: ::core::option::Option<Field1>,
        #[prost(string, repeated, tag = "3")]
        pub field2: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        #[prost(enumeration = "EnumOk", tag = "4")]
        pub field3: i32,
    }
    #[derive(
        serde::Serialize,
        serde::Deserialize,
        Clone,
        PartialEq,
        Eq,
        Hash,
        ::prost::Message,
        bevy_reflect::Reflect,
    )]
    pub struct Field1 {
        #[prost(string, tag = "1")]
        pub fname: ::prost::alloc::string::String,
        #[prost(int32, tag = "2")]
        pub fcount: i32,
    }

    #[derive(
        serde::Serialize,
        serde::Deserialize,
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration,
        bevy_reflect::Reflect,
    )]
    #[repr(i32)]
    pub enum EnumOk {
        Ok0 = 0,
        Ok1 = 1,
    }
    impl EnumOk {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                Self::Ok0 => "Ok0",
                Self::Ok1 => "Ok1",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "Ok0" => Some(Self::Ok0),
                "Ok1" => Some(Self::Ok1),
                _ => None,
            }
        }
    }

    impl clap::Args for Field1 {
        fn augment_args(cmd: clap::Command) -> clap::Command {
            impl_augment_args(cmd, Self::type_info())
        }

        fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
            Self::augment_args(cmd)
        }
    }

    impl clap::FromArgMatches for Field1 {
        fn from_arg_matches(
            matches: &clap::ArgMatches,
        ) -> ::std::result::Result<Self, clap::Error> {
            // 🎯 SERDE APPROACH: Use JSON deserialization instead of reflection downcasting
            impl_from_arg_matches(matches)
        }

        fn update_from_arg_matches(
            &mut self,
            matches: &clap::ArgMatches,
        ) -> ::std::result::Result<(), clap::Error> {
            // For update, we deserialize into a new instance and replace self
            *self = impl_from_arg_matches(matches)?;
            Ok(())
        }
    }

    impl clap::Args for HelloRequest2 {
        fn augment_args(cmd: clap::Command) -> clap::Command {
            impl_augment_args(cmd, Self::type_info())
        }

        fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
            Self::augment_args(cmd)
        }
    }
    impl clap::FromArgMatches for HelloRequest2 {
        fn from_arg_matches(
            matches: &clap::ArgMatches,
        ) -> ::std::result::Result<Self, clap::Error> {
            // 🎯 SERDE APPROACH: Use JSON deserialization for complex nested structs
            impl_from_arg_matches(matches)
        }

        fn update_from_arg_matches(
            &mut self,
            matches: &clap::ArgMatches,
        ) -> ::std::result::Result<(), clap::Error> {
            // For update, we deserialize into a new instance and replace self
            *self = impl_from_arg_matches(matches)?;
            Ok(())
        }
    }

    #[test]
    fn test_serde_based_struct_population() {
        println!("=== Testing Serde-Based Struct Population ===");

        // Test simple Field1 struct
        let cmd = Field1::augment_args(Command::new("test"));
        let matches = cmd
            .try_get_matches_from(["test", "--fname", "Hello Serde", "--fcount", "42"])
            .unwrap();

        let field1 = Field1::from_arg_matches(&matches).unwrap();
        assert_eq!(field1.fname, "Hello Serde");
        assert_eq!(field1.fcount, 42);
        println!("✅ Field1 populated via serde: {:?}", field1);

        // Test complex HelloRequest2 with nested Option<Field1>
        let cmd = HelloRequest2::augment_args(Command::new("test"));
        let matches = cmd
            .try_get_matches_from([
                "test",
                "--name",
                "Serde World",
                "--field1.fname",
                "Nested Serde",
                "--field1.fcount",
                "100",
                "--field2",
                "f2",
                "--field3",
                "1",
            ])
            .unwrap();

        let request = HelloRequest2::from_arg_matches(&matches).unwrap();
        assert_eq!(request.name, "Serde World");

        // Check if nested Option<Field1> was populated correctly
        let f1 = request.field1.unwrap();
        assert_eq!(f1.fname, "Nested Serde");
        assert_eq!(f1.fcount, 100);
        assert_eq!(request.field2, vec!["f2"]);
        assert_eq!(request.field3, EnumOk::Ok1 as i32);
        clear_all_stored_type_info();
    }

    #[test]
    fn test_reflection_based_parsing() {
        println!("=== Testing Reflection-Based Parsing ===");

        // Test Field1 parsing
        let mut cmd = Field1::augment_args(Command::new("test"));
        let matches = cmd
            .clone()
            .try_get_matches_from(vec!["test", "--fname", "hello", "--fcount", "42"])
            .unwrap();

        let field1 = Field1::from_arg_matches(&matches).unwrap();
        assert_eq!(field1.fname, "hello");
        assert_eq!(field1.fcount, 42);
        println!("✓ Field1 parsing works with generic helper");

        // Test HelloRequest2 with nested Field1
        cmd = HelloRequest2::augment_args(Command::new("test"));
        let matches = cmd
            .clone()
            .try_get_matches_from(vec![
                "test",
                "--name",
                "world",
                "--field1.fname",
                "nested",
                "--field1.fcount",
                "123",
                "--field2",
                "item1",
                "--field2",
                "item2",
                "--field3",
                "0",
            ])
            .unwrap();

        let hello_req = HelloRequest2::from_arg_matches(&matches).unwrap();
        assert_eq!(hello_req.name, "world");
        assert!(hello_req.field1.is_some());
        let nested_field1 = hello_req.field1.unwrap();
        assert_eq!(nested_field1.fname, "nested");
        assert_eq!(nested_field1.fcount, 123);
        assert_eq!(hello_req.field2, vec!["item1", "item2"]);
        assert_eq!(hello_req.field3, EnumOk::Ok0 as i32);
        println!("✓ HelloRequest2 with nested Field1 parsing works with generic helper");

        println!("✓ All tests passed - recursive parsing with generic helper works!");
        clear_all_stored_type_info();
    }

    fn clear_all_stored_type_info() {
        TYPE_INFO_STORE.with_borrow_mut(|store| {
            store.clear();
        });
    }

    #[test]
    fn test_store_type_info() {
        // Test the scenario that was causing the panic
        // Store type info as it would be stored during actual processing
        store_field_type_for_struct(
            "hwgencli::helloworld::HelloRequest2",
            "name",
            StoredFieldType::String,
        );
        store_field_type_for_struct(
            "hwgencli::helloworld::HelloRequest2",
            "field1",
            StoredFieldType::OptionCustom(
                "core::option::Option<hwgencli::helloworld::Field1>".to_string(),
            ),
        );
        store_field_type_for_struct(
            "hwgencli::helloworld::HelloRequest2",
            "field2",
            StoredFieldType::VecString,
        );
        store_field_type_for_struct(
            "hwgencli::helloworld::HelloRequest2",
            "field3",
            StoredFieldType::I32,
        );

        // Store nested field info as it would be stored for Field1 with prefix
        store_field_type_for_struct(
            "hwgencli::helloworld::Field1",
            "field1.fname",
            StoredFieldType::String,
        );
        store_field_type_for_struct(
            "hwgencli::helloworld::Field1",
            "field1.fcount",
            StoredFieldType::I32,
        );

        set_current_struct_name("hwgencli::helloworld::HelloRequest2");
        // Test that we can retrieve the stored types
        assert!(get_field_type_for_struct("hwgencli::helloworld::HelloRequest2", "name").is_some());
        assert!(
            get_field_type_for_struct("hwgencli::helloworld::HelloRequest2", "field1").is_some()
        );
        assert!(
            get_field_type_for_struct("hwgencli::helloworld::HelloRequest2", "field2").is_some()
        );
        assert!(
            get_field_type_for_struct("hwgencli::helloworld::HelloRequest2", "field3").is_some()
        );

        // Test the critical case that was failing: nested field lookup
        // This should work with our smart fallback logic even without current struct context
        let nested_fname_type = get_field_type("field1.fname");
        assert!(
            nested_fname_type.is_some(),
            "Should find type for nested field field1.fname"
        );

        let nested_fcount_type = get_field_type("field1.fcount");
        assert!(
            nested_fcount_type.is_some(),
            "Should find type for nested field field1.fcount"
        );

        // Verify the types are correct
        match nested_fname_type.unwrap() {
            StoredFieldType::String => {}
            other => panic!("Expected String type for field1.fname, got {:?}", other),
        }

        match nested_fcount_type.unwrap() {
            StoredFieldType::I32 => {}
            other => panic!("Expected I32 type for field1.fcount, got {:?}", other),
        }

        // Test edge cases: non-existent fields should return None
        assert!(get_field_type("completely_nonexistent_field").is_none());
        assert!(get_field_type("field999.nonexistent").is_none());
        clear_current_struct_name();

        // Test with current struct context set
        set_current_struct_name("hwgencli::helloworld::Field1");
        let fname_with_context = get_field_type("field1.fname");
        assert!(
            fname_with_context.is_some(),
            "Should find type with current struct context"
        );
        clear_current_struct_name();

        // Print debug info to verify what's stored
        println!("✅ All nested field lookups work correctly!");
        TYPE_INFO_STORE.with_borrow(|store| {
            println!("🔍 Stored type info:");
            for (key, value) in store.iter() {
                println!("  {} -> {:?}", key, value);
            }
        });

        clear_all_stored_type_info();
    }
}
