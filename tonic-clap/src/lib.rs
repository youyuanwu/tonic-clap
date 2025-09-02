mod clap;
pub use clap::{impl_augment_args, impl_from_arg_matches};

pub use tonic_clap_macros::TonicClap;

pub mod visit;

pub mod arg;

/// Common boxed error.
pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
