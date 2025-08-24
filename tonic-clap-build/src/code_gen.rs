use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream;
use prost_build::Service;
use quote::quote;

pub struct ServiceGenerator {
    services: Vec<prost_build::Service>,
    _args_code: TokenStream,
}

impl ServiceGenerator {
    pub fn new() -> Self {
        ServiceGenerator {
            _args_code: TokenStream::default(),
            services: Vec::new(),
        }
    }
}

impl prost_build::ServiceGenerator for ServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, _buf: &mut String) {
        // collect all services
        self.services.push(service);
    }

    fn finalize(&mut self, buf: &mut String) {
        // skip empty pkg
        if self.services.is_empty() {
            return;
        }

        let builder = CodeGenBuilder {};

        let mut total_code = TokenStream::new();

        let svc_enum = builder.generate_cmd_services_enum(&self.services);
        total_code.extend(svc_enum);
        // generate svc enum
        // generate method enum
        for svc in &self.services {
            let method_enum = builder.generate_svc_method_enum(svc);
            total_code.extend(method_enum);
        }
        let result = quote! {
            pub mod cli{
                #total_code
            }
        };
        buf.push_str(&result.to_string());
        self.services.clear();
    }
}

struct CodeGenBuilder {}

impl CodeGenBuilder {
    pub fn generate_cmd_services_enum(&self, services: &Vec<Service>) -> TokenStream {
        let mut svc_enum_stream = TokenStream::new();
        let mut svc_call_stream = TokenStream::new();
        for svc in services {
            let svc_name = quote::format_ident!("{}", svc.name);
            let svc_enum_name = quote::format_ident!("{}Commands", svc.name);
            let enum_tokens = quote! {
                    #[command(subcommand)]
                    #svc_name(#svc_enum_name),
            };
            svc_enum_stream.extend(enum_tokens);

            let svc_call_tokens = quote! {
                Self::#svc_name(cmd) => cmd.execute(ch, json_data).await,
            };
            svc_call_stream.extend(svc_call_tokens);
        }
        // TODO: execution fn.
        let exe_fn = quote! {
            impl CommandServices {
                pub async fn execute(
                    self,
                    ch: tonic::transport::Channel,
                    json_data: Option<String>,
                ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status> {

                    match self {
                        #svc_call_stream
                    }
                }
            }
        };

        quote! {
            #[derive(clap::Subcommand, Debug)]
            pub enum CommandServices {
                #svc_enum_stream
            }

            #exe_fn
        }
    }

    /// Generate the clap enum for a service containing all methods.
    pub fn generate_svc_method_enum(&self, svc: &Service) -> TokenStream {
        let svc_enum_name = quote::format_ident!("{}Commands", svc.name);
        let mut method_enum_stream = TokenStream::new();
        let mut method_call_stream = TokenStream::new();
        for m in &svc.methods {
            if m.server_streaming || m.client_streaming {
                // skip streaming methods for now.
                continue;
            }
            let method_enum_val = quote::format_ident!("{}", m.name.to_upper_camel_case());
            // type in the same pkg
            // it is in the outer mod.
            let input_type: syn::Path =
                syn::parse_str(&format!("super::{}", m.input_type)).unwrap();
            let method_name = quote::format_ident!("{}", m.name);

            let enum_tokens = quote! {
                #method_enum_val(#input_type),
            };
            method_enum_stream.extend(enum_tokens);

            // For now if json data is present ignore args.
            // TODO: implement merging of json data with args.
            let method_call = quote! {
                #svc_enum_name::#method_enum_val(val) => {
                    let request: #input_type = match json_data {
                        Some(data) => serde_json::from_str(&data).unwrap(),
                        None => val,
                    };
                    Ok(Box::new(c.#method_name(request).await?.into_inner()))
                }
            };
            method_call_stream.extend(method_call);
        }

        // Generate execute function.
        let client_name = quote::format_ident!("{}Client", svc.name);
        let client_mod_name = quote::format_ident!("{}_client", svc.name.to_snake_case());
        let exe_fn = quote! {
            impl #svc_enum_name {
                async fn execute(
                    self,
                    ch: tonic::transport::Channel,
                    json_data: Option<String>,
                ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status> {
                    let mut c = super::#client_mod_name::#client_name::new(ch);
                    match self {
                        #method_call_stream
                    }
                }
            }
        };

        quote! {
            #[derive(clap::Subcommand, Debug)]
            pub enum #svc_enum_name {
                #method_enum_stream
            }

            #exe_fn
        }
    }
}
