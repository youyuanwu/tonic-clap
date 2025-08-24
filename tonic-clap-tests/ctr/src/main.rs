pub mod containerd {
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
    }
}

pub mod google {
    pub mod protobuf {
        tonic::include_proto!("google.protobuf");
    }
}

use clap::Parser;

#[allow(clippy::large_enum_variant)]
#[derive(clap::Subcommand, Debug)]
pub enum CombinedArgs {
    #[clap(flatten)]
    Services(containerd::services::containers::v1::cli::CommandServices),
    #[clap(flatten)]
    Content(containerd::services::content::v1::cli::CommandServices),
}

pub type Args = tonic_clap::arg::DefaultArgs<CombinedArgs>;

fn main() {
    let args = Args::parse();
    println!("Args: {:?}", args);
}
