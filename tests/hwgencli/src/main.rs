include!("../gen/helloworld.rs");

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: Option<String>,

    #[arg(short, long)]
    json_data: Option<String>,

    #[command(subcommand)]
    command: CommandServices,
}

async fn connect(url: String) -> tonic::transport::Channel {
    let ep = tonic::transport::Endpoint::from_shared(url).unwrap();
    ep.connect().await.unwrap()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = Args::parse();

    // println!("Debug : {:?}", args);

    let ch = connect(args.url.unwrap()).await;

    let resp = args
        .command
        .execute(ch, args.json_data)
        .await
        .expect("request failed");
    println!("RESPONSE={:?}", resp);
}
