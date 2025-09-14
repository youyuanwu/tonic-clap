pub mod cmd;

use clap::Parser;

pub type Args = tonic_clap::arg::DefaultArgs<cmd::CombinedArgs>;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    println!("Args: {:?}", args);
    args.run_main().await.unwrap();
}
