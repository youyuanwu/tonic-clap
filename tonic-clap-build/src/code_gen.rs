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
        let builder = CodeGenBuilder {};

        let svc_enum = builder.generate_cmd_services_enum(&self.services);
        buf.push_str(svc_enum.to_string().as_str());
        // generate svc enum
        // generate method enum
        for svc in &self.services {
            let method_enum = builder.generate_svc_method_enum(svc);
            buf.push_str(&method_enum.to_string());
        }
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
                    &self,
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
            let method_enum_val = quote::format_ident!("{}", m.name.to_upper_camel_case());
            let enum_tokens = quote! {
                #method_enum_val,
            };
            method_enum_stream.extend(enum_tokens);

            // execution branch
            let request = quote::format_ident!("{}", m.input_type);
            let method_name = quote::format_ident!("{}", m.name);
            let method_call = quote! {
                #svc_enum_name::#method_enum_val => {
                    let request: #request = match json_data {
                        Some(data) => serde_json::from_str(&data).unwrap(),
                        None => Default::default(),
                    };
                    Ok(Box::new(c.#method_name(request).await?))
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
                    &self,
                    ch: tonic::transport::Channel,
                    json_data: Option<String>,
                ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status> {
                    let mut c = #client_mod_name::#client_name::new(ch);
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
