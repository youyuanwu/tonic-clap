// This file is @generated by prost-build.
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloRequest2 {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub field1: ::core::option::Option<Field1>,
    #[prost(string, repeated, tag = "3")]
    pub field2: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(enumeration = "EnumOk", tag = "4")]
    pub field3: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Field1 {
    #[prost(string, tag = "1")]
    pub fname: ::prost::alloc::string::String,
    #[prost(int32, tag = "2")]
    pub fcount: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloReply {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloReply2 {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum EnumOk {
    Ok1 = 0,
    Ok2 = 1,
}
impl EnumOk {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::Ok1 => "Ok1",
            Self::Ok2 => "Ok2",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Ok1" => Some(Self::Ok1),
            "Ok2" => Some(Self::Ok2),
            _ => None,
        }
    }
}
/// Generated client implementations.
pub mod greeter_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// The greeting service definition.
    #[derive(Debug, Clone)]
    pub struct GreeterClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl GreeterClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> GreeterClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::Body>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> GreeterClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::Body>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            GreeterClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Sends a greeting
        pub async fn say_hello(
            &mut self,
            request: impl tonic::IntoRequest<super::HelloRequest>,
        ) -> std::result::Result<tonic::Response<super::HelloReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/helloworld.Greeter/SayHello",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("helloworld.Greeter", "SayHello"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn say_hello2(
            &mut self,
            request: impl tonic::IntoRequest<super::HelloRequest2>,
        ) -> std::result::Result<tonic::Response<super::HelloReply2>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/helloworld.Greeter/SayHello2",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("helloworld.Greeter", "SayHello2"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod greeter2_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct Greeter2Client<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl Greeter2Client<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> Greeter2Client<T>
    where
        T: tonic::client::GrpcService<tonic::body::Body>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> Greeter2Client<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::Body>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            Greeter2Client::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// Sends a greeting
        pub async fn say_hello(
            &mut self,
            request: impl tonic::IntoRequest<super::HelloRequest>,
        ) -> std::result::Result<tonic::Response<super::HelloReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/helloworld.Greeter2/SayHello",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("helloworld.Greeter2", "SayHello"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn say_hello2(
            &mut self,
            request: impl tonic::IntoRequest<super::HelloRequest2>,
        ) -> std::result::Result<tonic::Response<super::HelloReply2>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/helloworld.Greeter2/SayHello2",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("helloworld.Greeter2", "SayHello2"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn say_hello3(
            &mut self,
            request: impl tonic::IntoRequest<super::google::protobuf::Empty>,
        ) -> std::result::Result<tonic::Response<super::HelloReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/helloworld.Greeter2/SayHello3",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("helloworld.Greeter2", "SayHello3"));
            self.inner.unary(req, path, codec).await
        }
    }
}
pub mod cli {
    #[derive(clap::Subcommand, Debug)]
    pub enum CommandServices {
        #[command(subcommand)]
        Greeter(GreeterCommands),
        #[command(subcommand)]
        Greeter2(Greeter2Commands),
    }
    impl CommandServices {
        pub async fn execute(
            &self,
            ch: tonic::transport::Channel,
            json_data: Option<String>,
        ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status> {
            match self {
                Self::Greeter(cmd) => cmd.execute(ch, json_data).await,
                Self::Greeter2(cmd) => cmd.execute(ch, json_data).await,
            }
        }
    }
    #[derive(clap::Subcommand, Debug)]
    pub enum GreeterCommands {
        SayHello,
        SayHello2,
    }
    impl GreeterCommands {
        async fn execute(
            &self,
            ch: tonic::transport::Channel,
            json_data: Option<String>,
        ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status> {
            let mut c = super::greeter_client::GreeterClient::new(ch);
            match self {
                GreeterCommands::SayHello => {
                    let request: super::HelloRequest = match json_data {
                        Some(data) => serde_json::from_str(&data).unwrap(),
                        None => Default::default(),
                    };
                    Ok(Box::new(c.say_hello(request).await?))
                }
                GreeterCommands::SayHello2 => {
                    let request: super::HelloRequest2 = match json_data {
                        Some(data) => serde_json::from_str(&data).unwrap(),
                        None => Default::default(),
                    };
                    Ok(Box::new(c.say_hello2(request).await?))
                }
            }
        }
    }
    #[derive(clap::Subcommand, Debug)]
    pub enum Greeter2Commands {
        SayHello,
        SayHello2,
        SayHello3,
    }
    impl Greeter2Commands {
        async fn execute(
            &self,
            ch: tonic::transport::Channel,
            json_data: Option<String>,
        ) -> Result<Box<dyn std::fmt::Debug>, tonic::Status> {
            let mut c = super::greeter2_client::Greeter2Client::new(ch);
            match self {
                Greeter2Commands::SayHello => {
                    let request: super::HelloRequest = match json_data {
                        Some(data) => serde_json::from_str(&data).unwrap(),
                        None => Default::default(),
                    };
                    Ok(Box::new(c.say_hello(request).await?))
                }
                Greeter2Commands::SayHello2 => {
                    let request: super::HelloRequest2 = match json_data {
                        Some(data) => serde_json::from_str(&data).unwrap(),
                        None => Default::default(),
                    };
                    Ok(Box::new(c.say_hello2(request).await?))
                }
                Greeter2Commands::SayHello3 => {
                    let request: super::google::protobuf::Empty = match json_data {
                        Some(data) => serde_json::from_str(&data).unwrap(),
                        None => Default::default(),
                    };
                    Ok(Box::new(c.say_hello3(request).await?))
                }
            }
        }
    }
}
