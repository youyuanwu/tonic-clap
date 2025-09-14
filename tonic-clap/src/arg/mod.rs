// default arg for tonic

use clap::{Args, Parser, Subcommand};

#[cfg(feature = "openssl")]
pub mod openssl;

#[derive(Args, Debug)]
pub struct CommonArgs {
    /// JSON data to convert to proto payload. Ignored when options are specified.
    #[arg(short, long)]
    pub json_data: Option<String>,

    /// Do not send the request. Only prints the args.
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Args, Debug)]
pub struct TcpArgs<Sub>
where
    Sub: clap::Subcommand + std::fmt::Debug,
{
    /// The URL to send the request to.
    /// Example: http://localhost:8080
    #[arg(short, long, required = true)]
    pub url: String,

    #[command(flatten)]
    pub common: CommonArgs,

    #[command(subcommand)]
    pub command: Sub,
}

impl<Sub> TcpArgs<Sub>
where
    Sub: clap::Subcommand + std::fmt::Debug,
{
    pub fn make_channel(self) -> Result<CmdCtx<Sub>, crate::Error> {
        let ep = tonic::transport::Endpoint::from_shared(self.url)?;
        Ok(CmdCtx {
            channel: ep.connect_lazy(),
            common: self.common,
            cmd: self.command,
        })
    }
}

#[derive(Args, Debug)]
pub struct UdsArgs<Sub>
where
    Sub: clap::Subcommand + std::fmt::Debug,
{
    /// The URL to send the request to.
    /// - unix:relative_path
    /// - unix:///absolute_path
    #[arg(short, long, required = true)]
    pub url: String,

    #[command(flatten)]
    pub common: CommonArgs,

    #[command(subcommand)]
    pub command: Sub,
}

impl<Sub> UdsArgs<Sub>
where
    Sub: clap::Subcommand + std::fmt::Debug,
{
    pub fn make_channel(self) -> Result<CmdCtx<Sub>, crate::Error> {
        let ep = tonic::transport::Endpoint::from_shared(self.url)?;
        Ok(CmdCtx {
            channel: ep.connect_lazy(),
            common: self.common,
            cmd: self.command,
        })
    }
}

#[cfg(feature = "openssl")]
#[derive(Args, Debug)]
pub struct SslArgs<Sub>
where
    Sub: clap::Subcommand + std::fmt::Debug,
{
    #[command(flatten)]
    pub ssl: openssl::OpensslArgs,

    #[command(flatten)]
    pub common: CommonArgs,

    #[command(subcommand)]
    pub command: Sub,
}

#[derive(Subcommand, Debug)]
pub enum TransportMode<Sub>
where
    Sub: clap::Subcommand + std::fmt::Debug,
{
    /// Tcp without ssl
    Tcp(TcpArgs<Sub>),
    #[cfg(feature = "openssl")]
    /// Tcp with ssl
    TcpSsl(SslArgs<Sub>),
    /// Unix domain socket. Only works on linux.
    Uds(UdsArgs<Sub>),
}

impl<Sub> TransportMode<Sub>
where
    Sub: clap::Subcommand + std::fmt::Debug,
{
    pub fn make_channel(self) -> Result<CmdCtx<Sub>, crate::Error> {
        let ctx = match self {
            TransportMode::Tcp(tcp) => tcp.make_channel()?,
            #[cfg(feature = "openssl")]
            TransportMode::TcpSsl(ssl) => CmdCtx {
                channel: ssl.ssl.make_channel()?,
                common: ssl.common,
                cmd: ssl.command,
            },
            TransportMode::Uds(uds) => uds.make_channel()?,
        };
        Ok(ctx)
    }
}

/// Default arguments for tonic.
/// User may write their own for other usecases.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct DefaultArgs<Sub>
where
    Sub: clap::Subcommand + std::fmt::Debug,
{
    #[command(subcommand)]
    pub transport: TransportMode<Sub>,
}

/// Stuff to return for user to call
pub struct CmdCtx<Sub> {
    pub channel: tonic::transport::Channel,
    pub common: CommonArgs,
    pub cmd: Sub,
}

/// Each service or combined service should implement this trait
#[allow(async_fn_in_trait)]
pub trait ExecuteCmd {
    async fn execute(
        self,
        channel: tonic::transport::Channel,
        json_data: Option<String>,
    ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status>;
}

impl<Sub> DefaultArgs<Sub>
where
    Sub: clap::Subcommand + std::fmt::Debug + crate::arg::ExecuteCmd,
{
    // Default main function to run a CLI app built with `tonic-clap`.
    pub async fn run_main(self) -> Result<(), crate::Error> {
        let ctx = self.transport.make_channel()?;
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
}
