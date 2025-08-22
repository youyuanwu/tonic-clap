use tonic_clap::ClapArgs;
use clap::Parser;

// Example nested prost-generated struct
#[derive(
    serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash, 
    prost::Message, ClapArgs, Debug
)]
pub struct Details {
    #[prost(string, tag = "1")]
    pub description: String,
    #[prost(int32, tag = "2")]
    pub priority: i32,
}

// Example main prost-generated struct that uses nested struct
#[derive(
    serde::Serialize, serde::Deserialize, Clone, PartialEq, Eq, Hash, 
    prost::Message, ClapArgs, Debug
)]
pub struct MyRequest {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(message, optional, tag = "2")]
    pub details: Option<Details>,  // This now automatically works!
    #[prost(string, repeated, tag = "3")]
    pub tags: Vec<String>,
    #[prost(int32, tag = "4")]
    pub count: i32,
}

#[derive(Parser, Debug)]
#[command(name = "clap-args-demo")]
#[command(about = "Demo of improved ClapArgs derive macro")]
struct Cli {
    #[command(flatten)]
    request: MyRequestArg,
}

fn main() {
    let cli = Cli::parse();
    
    // Convert the clap args to the original prost struct
    let request: MyRequest = cli.request.into();
    
    println!("Generated request:");
    println!("{:#?}", request);
    
    // Also demonstrate the apply method
    let mut manual_request = MyRequest::default();
    let args = MyRequestArg {
        name: Some("Manual".to_string()),
        details: Some(DetailsArg {
            description: Some("Important task".to_string()),
            priority: Some("5".to_string()),
        }),
        tags: vec!["tag1".to_string()],
        count: Some("42".to_string()),
    };
    
    args.apply(&mut manual_request);
    println!("\nManual request after apply:");
    println!("{:#?}", manual_request);
}

// Test that shows the generated structs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_automatic_nested_args() {
        // Test that the macro automatically generated args for nested structs
        let details_args = DetailsArg {
            description: Some("Test details".to_string()),
            priority: Some("3".to_string()),
        };
        
        let args = MyRequestArg {
            name: Some("test".to_string()),
            details: Some(details_args),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
            count: Some("10".to_string()),
        };
        
        // Test From conversion
        let request: MyRequest = args.clone().into();
        assert_eq!(request.name, "test");
        assert_eq!(request.tags, vec!["tag1", "tag2"]);
        assert_eq!(request.count, 10);
        
        // Test nested field conversion
        assert!(request.details.is_some());
        let details = request.details.unwrap();
        assert_eq!(details.description, "Test details");
        assert_eq!(details.priority, 3);
        
        // Test apply method
        let mut request2 = MyRequest::default();
        args.apply(&mut request2);
        assert_eq!(request2.name, "test");
        assert_eq!(request2.count, 10);
        assert!(request2.details.is_some());
        
        println!("Automatic nested args work correctly!");
    }
}
