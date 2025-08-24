use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Combined derive macro that implements both Args and FromArgMatches
#[proc_macro_derive(TonicClap)]
pub fn derive_tonic_clap(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        impl clap::Args for #name {
            fn augment_args(cmd: clap::Command) -> clap::Command {
                use bevy_reflect::Typed;
                tonic_clap::impl_augment_args(cmd, Self::type_info())
            }

            fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
                Self::augment_args(cmd)
            }
        }

        impl clap::FromArgMatches for #name {
            fn from_arg_matches(
                matches: &clap::ArgMatches,
            ) -> ::std::result::Result<Self, clap::Error> {
                tonic_clap::impl_from_arg_matches(matches)
            }

            fn update_from_arg_matches(
                &mut self,
                matches: &clap::ArgMatches,
            ) -> ::std::result::Result<(), clap::Error> {
                *self = tonic_clap::impl_from_arg_matches(matches)?;
                Ok(())
            }
        }
    };

    TokenStream::from(expanded)
}
