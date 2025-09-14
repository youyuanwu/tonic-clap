pub mod containerd {
    pub mod v1 {
        pub mod types {
            tonic::include_proto!("containerd.v1.types");
        }
    }
    pub mod types {
        tonic::include_proto!("containerd.types");
    }
    pub mod services {
        pub mod containers {
            pub mod v1 {
                tonic::include_proto!("containerd.services.containers.v1");
            }
        }
        pub mod content {
            pub mod v1 {
                tonic::include_proto!("containerd.services.content.v1");
            }
        }
        pub mod diff {
            pub mod v1 {
                tonic::include_proto!("containerd.services.diff.v1");
            }
        }
        pub mod events {
            pub mod v1 {
                tonic::include_proto!("containerd.services.events.v1");
            }
        }
        pub mod images {
            pub mod v1 {
                tonic::include_proto!("containerd.services.images.v1");
            }
        }
        pub mod introspection {
            pub mod v1 {
                tonic::include_proto!("containerd.services.introspection.v1");
            }
        }
        pub mod leases {
            pub mod v1 {
                tonic::include_proto!("containerd.services.leases.v1");
            }
        }
        pub mod namespaces {
            pub mod v1 {
                tonic::include_proto!("containerd.services.namespaces.v1");
            }
        }
        pub mod sandbox {
            pub mod v1 {
                tonic::include_proto!("containerd.services.sandbox.v1");
            }
        }
        pub mod snapshots {
            pub mod v1 {
                tonic::include_proto!("containerd.services.snapshots.v1");
            }
        }
        pub mod tasks {
            pub mod v1 {
                tonic::include_proto!("containerd.services.tasks.v1");
            }
        }
        pub mod transfer {
            pub mod v1 {
                tonic::include_proto!("containerd.services.transfer.v1");
            }
        }
        pub mod version {
            pub mod v1 {
                tonic::include_proto!("containerd.services.version.v1");
            }
        }
    }
}

pub mod google {
    pub mod protobuf {
        tonic::include_proto!("google.protobuf");
    }
    pub mod rpc {
        tonic::include_proto!("google.rpc");
    }
}

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
