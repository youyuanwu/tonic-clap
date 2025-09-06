pub mod helloworld {
    include!("../gen/helloworld.rs");
}
pub mod google {
    pub mod protobuf {
        include!("../gen/google.protobuf.rs");
    }
}

pub type HWArgs = tonic_clap::arg::DefaultArgs<helloworld::cli::CommandServices>;

pub mod server;

#[cfg(test)]
mod ref_tests;

#[cfg(test)]
mod e2e_tests;

#[cfg(test)]
mod internal_tests;
