pub mod models {
    use clap::{Args, Command, FromArgMatches};

    // Test struct using derive macros
    #[derive(
        serde::Serialize,
        serde::Deserialize,
        Clone,
        PartialEq,
        Eq,
        Hash,
        ::prost::Message,
        bevy_reflect::Reflect,
        tonic_clap::TonicClap,
    )]
    pub struct TestWithDerived {
        #[prost(string, tag = "1")]
        pub name: ::prost::alloc::string::String,
        #[prost(int32, tag = "2")]
        pub count: i32,
    }

    #[test]
    fn test_derive_macro() {
        println!("=== Testing Derive Macro ===");

        // Test struct with derived implementations
        let cmd = TestWithDerived::augment_args(Command::new("test"));
        let matches = cmd
            .try_get_matches_from(["test", "--name", "Derived Test", "--count", "999"])
            .unwrap();

        let derived = TestWithDerived::from_arg_matches(&matches).unwrap();
        assert_eq!(derived.name, "Derived Test");
        assert_eq!(derived.count, 999);
        println!(
            "✅ TestWithDerived populated via derive macro: {:?}",
            derived
        );
    }
}
