// Example demonstrating pure reflection-based field setting without hardcoded field names

use clap::{Args, Command, FromArgMatches};
use tonic_clap::{impl_augment_args, impl_from_arg_matches};

use crate::helloworld::{EnumOk, Field1, HelloRequest2};

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
            use bevy_reflect::Typed;
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
