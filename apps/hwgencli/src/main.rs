pub mod helloworld {
    tonic::include_proto!("helloworld");
}
pub mod google {
    pub mod protobuf {
        tonic::include_proto!("google.protobuf");
    }
}

use clap::Parser;

/// Simple program to greet a person
pub type Args = tonic_clap_tests::HWArgs;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), tonic_clap::Error> {
    let args = Args::parse();
    args.run_main().await
}
