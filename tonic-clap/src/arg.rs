// default arg for tonic

use clap::Parser;

/// Default arguments for tonic.
/// User may write their own for other usecases.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct DefaultArgs<T>
where
    T: clap::Subcommand + std::fmt::Debug,
{
    #[arg(short, long)]
    pub url: Option<String>,

    #[arg(short, long)]
    pub json_data: Option<String>,

    #[command(subcommand)]
    pub command: T,
}
