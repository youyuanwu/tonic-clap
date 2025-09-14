pub mod cmd;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "ctr", version, about = "Containerd CLI tool", long_about = None)]
pub struct Args {
    #[command(flatten)]
    pub default_args: tonic_clap::arg::DefaultArgs<cmd::CombinedArgs>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();
    println!("Args: {:?}", args);
    args.default_args.run_main().await.unwrap();
}
