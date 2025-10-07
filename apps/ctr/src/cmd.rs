use crate::included::containerd;

#[allow(clippy::large_enum_variant)]
#[derive(clap::Subcommand, Debug)]
pub enum CombinedArgs {
    #[clap(flatten)]
    Services(containerd::services::containers::v1::cli::CommandServices),
    #[clap(flatten)]
    Content(containerd::services::content::v1::cli::CommandServices),
    #[clap(flatten)]
    Diff(containerd::services::diff::v1::cli::CommandServices),
    #[clap(flatten)]
    Events(containerd::services::events::v1::cli::CommandServices),
    #[clap(flatten)]
    Images(containerd::services::images::v1::cli::CommandServices),
    #[clap(flatten)]
    Introspection(containerd::services::introspection::v1::cli::CommandServices),
    #[clap(flatten)]
    Leases(containerd::services::leases::v1::cli::CommandServices),
    #[clap(flatten)]
    Namespaces(containerd::services::namespaces::v1::cli::CommandServices),
    #[clap(flatten)]
    Sandbox(containerd::services::sandbox::v1::cli::CommandServices),
    #[clap(flatten)]
    Snapshots(containerd::services::snapshots::v1::cli::CommandServices),
    #[clap(flatten)]
    Tasks(containerd::services::tasks::v1::cli::CommandServices),
    #[clap(flatten)]
    Transfer(containerd::services::transfer::v1::cli::CommandServices),
    #[clap(flatten)]
    Version(containerd::services::version::v1::cli::CommandServices),
}

impl tonic_clap::arg::ExecuteCmd for CombinedArgs {
    async fn execute(
        self,
        channel: tonic::transport::Channel,
        json_data: Option<String>,
    ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status> {
        match self {
            CombinedArgs::Services(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Content(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Diff(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Events(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Images(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Introspection(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Leases(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Namespaces(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Sandbox(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Snapshots(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Tasks(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Transfer(cmd) => cmd.execute(channel, json_data).await,
            CombinedArgs::Version(cmd) => cmd.execute(channel, json_data).await,
        }
    }
}
