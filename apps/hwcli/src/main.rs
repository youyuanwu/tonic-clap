use clap::Parser;

mod cligen;

/// Simple program to greet a person
pub type Args = tonic_clap::arg::DefaultArgs<cligen::CommandServices>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), tonic_clap::Error> {
    let args = Args::parse();
    args.run_main().await
}
