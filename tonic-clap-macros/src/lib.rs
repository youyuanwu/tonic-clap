use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

// /// Derive macro for implementing clap::Args trait using reflection-based approach
// #[proc_macro_derive(TonicArgs)]
// pub fn derive_tonic_args(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = &input.ident;

//     let expanded = quote! {
//         impl clap::Args for #name {
//             fn augment_args(cmd: clap::Command) -> clap::Command {
//                 crate::impl_augment_args(cmd, Self::type_info())
//             }

//             fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
//                 Self::augment_args(cmd)
//             }
//         }
//     };

//     TokenStream::from(expanded)
// }

// /// Derive macro for implementing clap::FromArgMatches trait using serde-based approach
// #[proc_macro_derive(TonicFromArgMatches)]
// pub fn derive_tonic_from_arg_matches(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = &input.ident;

//     let expanded = quote! {
//         impl clap::FromArgMatches for #name {
//             fn from_arg_matches(
//                 matches: &clap::ArgMatches,
//             ) -> ::std::result::Result<Self, clap::Error> {
//                 crate::impl_from_arg_matches(matches)
//             }

//             fn update_from_arg_matches(
//                 &mut self,
//                 matches: &clap::ArgMatches,
//             ) -> ::std::result::Result<(), clap::Error> {
//                 *self = crate::impl_from_arg_matches(matches)?;
//                 Ok(())
//             }
//         }
//     };

//     TokenStream::from(expanded)
// }

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

#[proc_macro_attribute]
pub fn duplicate_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemStruct);
    let original_ident = &input.ident;
    let new_ident = syn::Ident::new(&format!("{}Copy", original_ident), original_ident.span());

    let vis = &input.vis;
    let generics = &input.generics;

    // Strip attributes from each field
    let fields: Vec<syn::Field> = input
        .fields
        .iter()
        .map(|f| {
            let mut f = f.clone();
            f.attrs.clear(); // This removes all field-level attributes
            f
        })
        .collect();

    let struct_fields = match &input.fields {
        syn::Fields::Named(_) => quote! { { #(#fields),* } },
        syn::Fields::Unnamed(_) => quote! { ( #(#fields),* ); },
        syn::Fields::Unit => quote! { ; },
    };

    let expanded = quote! {
        #input

        #vis struct #new_ident #generics #struct_fields
    };

    TokenStream::from(expanded)
}

#[cfg(test)]
mod tests {}
