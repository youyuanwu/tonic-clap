use bevy_reflect::{TypeInfo, Typed};
use serde_json::Value;

pub fn impl_from_arg_matches<T>(matches: &clap::ArgMatches) -> Result<T, clap::Error>
where
    T: serde::de::DeserializeOwned + Typed,
{
    let type_info = T::type_info();
    let tree = crate::visit::TCFieldType::parse(type_info);

    let mut root_json = serde_json::Map::new();

    tree.visit_nested(&mut |prefix, field_name, field_type| {
        // Use the prefix, field_name, and field_type to extract values from matches
        assert!(field_type.is_primitive());
        let arg_name = if prefix.is_empty() {
            field_name.clone()
        } else {
            prefix.join(".") + "." + field_name
        };

        // Check if argument exists first
        if !matches.contains_id(&arg_name) {
            return;
        }

        // Extract the primitive value
        if let Some(field_value) = extract_primitive_value(matches, &arg_name, field_type) {
            // Set the value at the correct nested path
            let mut path_parts = prefix.to_vec();
            path_parts.push(field_name.to_string());
            set_nested_value(&mut root_json, &path_parts, field_value);
        }
    });

    // Convert the map to JSON Value
    let json_value = Value::Object(root_json);

    // Use serde to deserialize directly into the target struct
    match serde_json::from_value::<T>(json_value) {
        Ok(instance) => Ok(instance),
        Err(e) => Err(clap::Error::raw(
            clap::error::ErrorKind::ValueValidation,
            format!("Failed to deserialize struct from arguments: {}", e),
        )),
    }
}

// Helper function to extract primitive value from matches
fn extract_primitive_value(
    matches: &clap::ArgMatches,
    arg_name: &str,
    field_type: &crate::visit::TCFieldType,
) -> Option<Value> {
    use crate::visit::TCFieldType;

    match field_type {
        TCFieldType::String => {
            if let Ok(Some(value)) = matches.try_get_one::<String>(arg_name) {
                Some(Value::String(value.clone()))
            } else {
                None
            }
        }
        TCFieldType::I32 => {
            if let Ok(Some(value)) = matches.try_get_one::<i32>(arg_name) {
                Some(Value::Number(serde_json::Number::from(*value)))
            } else {
                None
            }
        }
        TCFieldType::I64 => {
            if let Ok(Some(value)) = matches.try_get_one::<i64>(arg_name) {
                Some(Value::Number(serde_json::Number::from(*value)))
            } else {
                None
            }
        }
        TCFieldType::F32 => {
            if let Ok(Some(value)) = matches.try_get_one::<f32>(arg_name) {
                serde_json::Number::from_f64(*value as f64).map(Value::Number)
            } else {
                None
            }
        }
        TCFieldType::F64 => {
            if let Ok(Some(value)) = matches.try_get_one::<f64>(arg_name) {
                serde_json::Number::from_f64(*value).map(Value::Number)
            } else {
                None
            }
        }
        TCFieldType::Bool => {
            if let Ok(Some(value)) = matches.try_get_one::<bool>(arg_name) {
                Some(Value::Bool(*value))
            } else if matches.get_flag(arg_name) {
                Some(Value::Bool(true))
            } else {
                None
            }
        }
        TCFieldType::Vec(_) => {
            // Handle vectors - for now assume Vec<String>
            if let Ok(Some(values)) = matches.try_get_many::<String>(arg_name) {
                let vec_values: Vec<String> = values.cloned().collect();
                let json_array: Vec<Value> = vec_values.into_iter().map(Value::String).collect();
                Some(Value::Array(json_array))
            } else {
                None
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
    tree.visit_nested(&mut |prefix, field_name, field_type| {
        assert!(field_type.is_primitive());
        let arg_name = if prefix.is_empty() {
            field_name.clone()
        } else {
            prefix.join(".") + "." + field_name
        };
        // Statics needed to bypass lifetime checks.
        let help_text = format!("Arg: {}", field_type.display_primitive_type());
        let value_name_static: &'static str = Box::leak(field_name.to_uppercase().into_boxed_str());
        let arg_name_static: &'static str = Box::leak(arg_name.clone().into_boxed_str());
        let help_text_static: &'static str = Box::leak(help_text.clone().into_boxed_str());
        let (value_parser, action) = field_type.get_clap_value_parse();
        let arg = clap::Arg::new(arg_name_static)
            .long(arg_name_static)
            .value_name(value_name_static)
            .help(help_text_static)
            .required(false)
            .action(action)
            .value_parser(value_parser.clone());
        args.push(arg);
    });
    for arg in args {
        cmd = cmd.arg(arg);
    }
    cmd
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
    }

    #[test]
    fn test_multi_level_nesting() {
        println!("=== Testing Multi-Level Nesting ===");

        // Create a test structure with 3+ levels of nesting
        #[derive(serde::Serialize, serde::Deserialize, bevy_reflect::Reflect)]
        struct Level3 {
            name: String,
            value: i32,
        }

        #[derive(serde::Serialize, serde::Deserialize, bevy_reflect::Reflect)]
        struct Level2 {
            level3: Option<Level3>,
            count: i32,
        }

        #[derive(serde::Serialize, serde::Deserialize, bevy_reflect::Reflect)]
        struct Level1 {
            level2: Option<Level2>,
            id: String,
        }

        #[derive(serde::Serialize, serde::Deserialize, bevy_reflect::Reflect)]
        struct MultiLevelStruct {
            level1: Option<Level1>,
            top: String,
        }

        impl clap::Args for MultiLevelStruct {
            fn augment_args(cmd: clap::Command) -> clap::Command {
                impl_augment_args(cmd, Self::type_info())
            }
            fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
                Self::augment_args(cmd)
            }
        }

        impl clap::FromArgMatches for MultiLevelStruct {
            fn from_arg_matches(
                matches: &clap::ArgMatches,
            ) -> ::std::result::Result<Self, clap::Error> {
                impl_from_arg_matches(matches)
            }
            fn update_from_arg_matches(
                &mut self,
                matches: &clap::ArgMatches,
            ) -> ::std::result::Result<(), clap::Error> {
                *self = impl_from_arg_matches(matches)?;
                Ok(())
            }
        }

        // Test with deep nesting: top -> level1 -> level2 -> level3 -> name/value
        let cmd = MultiLevelStruct::augment_args(clap::Command::new("test"));
        let matches = cmd
            .try_get_matches_from([
                "test",
                "--top",
                "root_value",
                "--level1.id",
                "level1_id",
                "--level1.level2.count",
                "42",
                "--level1.level2.level3.name",
                "deep_name",
                "--level1.level2.level3.value",
                "99",
            ])
            .unwrap();

        let result = MultiLevelStruct::from_arg_matches(&matches).unwrap();

        // Test JSON structure by serializing to see the nested structure
        let json = serde_json::to_value(&result).unwrap();
        println!(
            "Generated JSON structure: {}",
            serde_json::to_string_pretty(&json).unwrap()
        );

        // Verify the JSON has the correct nested structure
        assert!(json["level1"]["level2"]["level3"]["name"].as_str() == Some("deep_name"));
        assert!(json["level1"]["level2"]["level3"]["value"].as_i64() == Some(99));

        // Verify all levels are properly nested
        assert_eq!(result.top, "root_value");

        let level1 = result.level1.expect("level1 should be populated");
        assert_eq!(level1.id, "level1_id");

        let level2 = level1.level2.expect("level2 should be populated");
        assert_eq!(level2.count, 42);

        let level3 = level2.level3.expect("level3 should be populated");
        assert_eq!(level3.name, "deep_name");
        assert_eq!(level3.value, 99);

        println!("✅ JSON structure properly represents multi-level nesting!");
    }
}
