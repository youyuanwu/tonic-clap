use proc_macro2::TokenStream;
use quote::quote;

// use crate::{client, server};
//use prost_build::Service;

pub struct ServiceGenerator {
    //builder: Builder
}

impl ServiceGenerator {
    pub fn new() -> Self {
        ServiceGenerator {
          // builder,
          // clients: TokenStream::default(),
          // servers: TokenStream::default(),
      }
    }
}

impl prost_build::ServiceGenerator for ServiceGenerator {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        let builder = CodeGenBuilder {};
        let client_code = builder.generate_cli(&service);
        buf.push_str(client_code.to_string().as_str());
    }
}

struct CodeGenBuilder {}

impl CodeGenBuilder {
    pub fn generate_cli(&self, service: &prost_build::Service) -> TokenStream {
        let service_ident = quote::format_ident!("{}Cli", service.name);
        quote! {
          // ok cli
          pub struct #service_ident{}
        }
    }
}
