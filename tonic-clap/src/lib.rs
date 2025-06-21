use proc_macro::TokenStream;
// use proc_macro2::TokenStream;
use quote::quote;
use syn::{Field, ItemStruct, parse_macro_input};

#[proc_macro_attribute]
pub fn duplicate_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let original_ident = &input.ident;
    let new_ident = syn::Ident::new(&format!("{}Copy", original_ident), original_ident.span());

    let vis = &input.vis;
    let generics = &input.generics;

    // Strip attributes from each field
    let fields: Vec<Field> = input
        .fields
        .iter()
        .map(|f| {
            let mut f = f.clone();
            f.attrs.clear(); // This removes all field-level attributes
            // if let Type::Path(TypePath { path, .. }) = &f.ty {
            //     // TODO: map type to usual type.
            // };
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
