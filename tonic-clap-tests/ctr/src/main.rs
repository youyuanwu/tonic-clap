pub mod cmd;

use clap::Parser;

pub type Args = tonic_clap::arg::DefaultArgs<cmd::CombinedArgs>;

fn main() {
    let args = Args::parse();
    println!("Args: {:?}", args);
}
