use proc_macro::TokenStream;
// use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Field, Fields, GenericArgument, ItemStruct, PathArguments, Type,
    parse_macro_input,
};

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

#[proc_macro_derive(ClapArgs, attributes(clap_args, prefix))]
pub fn clap_args(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let original_name = &input.ident;
    
    // Check for prefix attribute
    let prefix = input.attrs.iter()
        .find_map(|attr| {
            if attr.path().is_ident("prefix") {
                // Extract the prefix value from the attribute
                attr.parse_args::<syn::LitStr>().ok().map(|lit| lit.value())
            } else {
                None
            }
        })
        .unwrap_or_default();
    
    let args_name = if prefix.is_empty() {
        syn::Ident::new(&format!("{}Arg", original_name), original_name.span())
    } else {
        // If there's a prefix, create a prefixed version
        syn::Ident::new(&format!("{}{}Arg", capitalize(&prefix), original_name), original_name.span())
    };

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            syn::Fields::Named(fields) => &fields.named,
            _ => panic!("Only structs with named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let mut field_definitions = Vec::new();
    let mut from_conversions = Vec::new();
    let mut apply_conversions = Vec::new();
    let mut nested_structs: Vec<proc_macro2::TokenStream> = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        // Check for field-level prefix attribute
        let field_prefix = field
            .attrs
            .iter()
            .find_map(|attr| {
                if attr.path().is_ident("prefix") {
                    // Handle #[prefix = "value"] syntax
                    if let syn::Meta::NameValue(meta_name_value) = &attr.meta {
                        if let syn::Expr::Lit(expr_lit) = &meta_name_value.value {
                            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                return Some(lit_str.value());
                            }
                        }
                    }
                    // Fallback: handle #[prefix("value")] syntax  
                    attr.parse_args::<syn::LitStr>().ok().map(|lit| lit.value())
                } else {
                    None
                }
            });

        match field_type {
            // Handle Option<Message> fields - these become flattened nested Args
            Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "Option" => {
                if let PathArguments::AngleBracketed(args) =
                    &type_path.path.segments.last().unwrap().arguments
                {
                    if let Some(GenericArgument::Type(inner_type)) = args.args.first() {
                        // Check if this is a message type (not primitive)
                        if is_message_type(inner_type) {
                            let type_name = get_type_name(inner_type);
                            
                            // If field has a prefix attribute, use generic wrapper approach
                            if let Some(field_prefix_val) = &field_prefix {
                                // Generate a prefixed wrapper struct - works for ANY nested type
                                generate_prefixed_fields_for_nested_type(
                                    &type_name, 
                                    field_prefix_val,
                                    &original_name.to_string(),
                                    &mut field_definitions,
                                    &mut from_conversions,
                                    &mut apply_conversions,
                                    &mut nested_structs
                                );
                                continue;
                            } else {
                                // No prefix attribute, use default nested handling
                                generate_prefixed_fields_for_nested_type(
                                    &type_name, 
                                    &field_name.to_string(),
                                    &original_name.to_string(),
                                    &mut field_definitions,
                                    &mut from_conversions,
                                    &mut apply_conversions,
                                    &mut nested_structs
                                );
                                continue;
                            }
                        }
                    }
                }
                // Handle Option<primitive> types
                field_definitions.push(quote! {
                    #[arg(long)]
                    pub #field_name: #field_type,
                });
                from_conversions.push(quote! {
                    #field_name: args.#field_name,
                });
                apply_conversions.push(quote! {
                    if let Some(value) = &self.#field_name {
                        target.#field_name = Some(value.clone());
                    }
                });
            }
            // Handle Vec<T> fields
            Type::Path(type_path) if type_path.path.segments.last().unwrap().ident == "Vec" => {
                let arg_name = if let Some(field_prefix) = &field_prefix {
                    format!("{}_{}", field_prefix, field_name)
                } else if prefix.is_empty() {
                    field_name.to_string()
                } else {
                    format!("{}_{}", prefix, field_name)
                };
                
                field_definitions.push(quote! {
                    #[arg(long = #arg_name)]
                    pub #field_name: Vec<String>,
                });
                from_conversions.push(quote! {
                    #field_name: args.#field_name,
                });
                apply_conversions.push(quote! {
                    if !self.#field_name.is_empty() {
                        target.#field_name = self.#field_name.clone();
                    }
                });
            }
            // Handle regular fields (String, i32, etc.)
            _ => {
                let arg_name = if let Some(field_prefix) = &field_prefix {
                    format!("{}_{}", field_prefix, field_name)
                } else if prefix.is_empty() {
                    field_name.to_string()
                } else {
                    format!("{}_{}", prefix, field_name)
                };
                
                field_definitions.push(quote! {
                    #[arg(long = #arg_name)]
                    pub #field_name: Option<String>,
                });

                // Handle different field types in conversions
                if is_string_type(field_type) {
                    from_conversions.push(quote! {
                        #field_name: args.#field_name.unwrap_or_default(),
                    });
                    apply_conversions.push(quote! {
                        if let Some(value) = &self.#field_name {
                            target.#field_name = value.clone();
                        }
                    });
                } else if is_i32_type(field_type) {
                    from_conversions.push(quote! {
                        #field_name: args.#field_name.and_then(|s| s.parse().ok()).unwrap_or_default(),
                    });
                    apply_conversions.push(quote! {
                        if let Some(value) = &self.#field_name {
                            if let Ok(parsed) = value.parse() {
                                target.#field_name = parsed;
                            }
                        }
                    });
                } else {
                    // Default handling for other types
                    from_conversions.push(quote! {
                        #field_name: args.#field_name.and_then(|s| s.parse().ok()).unwrap_or_default(),
                    });
                    apply_conversions.push(quote! {
                        if let Some(value) = &self.#field_name {
                            if let Ok(parsed) = value.parse() {
                                target.#field_name = parsed;
                            }
                        }
                    });
                }
            }
        }
    }

    let expanded = quote! {
        #[derive(clap::Args, Debug, Clone, Default)]
        pub struct #args_name {
            #(#field_definitions)*
        }

        impl From<#args_name> for #original_name {
            fn from(args: #args_name) -> Self {
                Self {
                    #(#from_conversions)*
                }
            }
        }

        impl #args_name {
            pub fn apply(&self, target: &mut #original_name) {
                #(#apply_conversions)*
            }
        }

        // Generate prefixed companion structs for nested types
        #(#nested_structs)*
    };

    TokenStream::from(expanded)
}

// Helper functions
fn is_message_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path.path.segments.last().unwrap();
            // Check if it's not a primitive type
            !matches!(
                last_segment.ident.to_string().as_str(),
                "String" | "i32" | "i64" | "u32" | "u64" | "f32" | "f64" | "bool"
            )
        }
        _ => false,
    }
}

fn get_type_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => type_path.path.segments.last().unwrap().ident.to_string(),
        _ => "Unknown".to_string(),
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn is_string_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => {
            let path_str = quote!(#type_path).to_string();
            path_str.contains("String") || path_str.contains("string")
        }
        _ => false,
    }
}

fn is_i32_type(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path.path.segments.last().unwrap().ident == "i32",
        _ => false,
    }
}

fn generate_prefixed_fields_for_nested_type(
    type_name: &str,
    field_prefix: &str,
    parent_name: &str,
    field_definitions: &mut Vec<proc_macro2::TokenStream>,
    from_conversions: &mut Vec<proc_macro2::TokenStream>,
    apply_conversions: &mut Vec<proc_macro2::TokenStream>,
    nested_structs: &mut Vec<proc_macro2::TokenStream>,
) {
    // Generate a prefixed companion struct that mirrors the original Args struct
    // but with prefixed field names
    
    let field_name = syn::Ident::new(field_prefix, proc_macro2::Span::call_site());
    let original_type = syn::Ident::new(type_name, proc_macro2::Span::call_site());
    let original_arg_type = syn::Ident::new(&format!("{}Arg", type_name), proc_macro2::Span::call_site());
    let prefixed_struct_name = syn::Ident::new(&format!("{}{}Arg", parent_name, capitalize(field_prefix)), proc_macro2::Span::call_site());
    
    // Generate the prefixed companion struct
    // This is a macro-based approach that creates individual prefixed fields
    let prefixed_struct = generate_prefixed_companion_struct(&prefixed_struct_name, &original_type, &original_arg_type, field_prefix);
    nested_structs.push(prefixed_struct);
    
    // Use the prefixed struct in the parent
    field_definitions.push(quote! {
        #[command(flatten)]
        pub #field_name: Option<#prefixed_struct_name>,
    });

    from_conversions.push(quote! {
        #field_name: args.#field_name.map(|nested| nested.into()),
    });

    apply_conversions.push(quote! {
        if let Some(nested) = &self.#field_name {
            target.#field_name = Some(nested.clone().into());
        }
    });
}

fn generate_prefixed_companion_struct(
    prefixed_name: &syn::Ident,
    original_type: &syn::Ident, 
    original_arg_type: &syn::Ident,
    prefix: &str
) -> proc_macro2::TokenStream {
    // Generate a struct that simply wraps the original Args type with prefix
    // The original Args type should already have ClapArgs derived, so we just flatten it
    
    quote! {
        #[derive(clap::Args, Debug, Clone, Default)]
        pub struct #prefixed_name {
            #[command(flatten)]
            pub inner: Option<#original_arg_type>,
        }

        impl From<#prefixed_name> for #original_type {
            fn from(args: #prefixed_name) -> Self {
                args.inner.map(|inner| inner.into()).unwrap_or_default()
            }
        }
    }
}

#[cfg(test)]
mod tests {}
