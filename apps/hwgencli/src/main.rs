pub mod helloworld {
    include!("../gen/helloworld.rs");
}
pub mod google {
    pub mod protobuf {
        include!("../gen/google.protobuf.rs");
    }
}

use clap::Parser;

/// Simple program to greet a person
pub type Args = tonic_clap::arg::DefaultArgs<helloworld::cli::CommandServices>;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), tonic_clap::Error> {
    let args = Args::parse();

    let ctx = args.transport.make_channel()?;

    if ctx.common.dry_run {
        println!("dry run: {:?}", ctx.cmd);
        return Ok(());
    }

    let resp = ctx
        .cmd
        .execute(ctx.channel, ctx.common.json_data)
        .await
        .expect("request failed");
    println!("{:?}", resp);
    Ok(())
}
