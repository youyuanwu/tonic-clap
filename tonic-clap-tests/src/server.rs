use tonic::{Request, Response, Status};

use crate::server::helloworld::*;

mod helloworld {
    tonic::include_proto!("helloworld");
}

mod google {
    pub mod protobuf {
        tonic::include_proto!("google.protobuf");
    }
}

pub struct GreeterImpl {}

impl GreeterImpl {
    pub fn new_svc() -> greeter_server::GreeterServer<Self> {
        greeter_server::GreeterServer::new(GreeterImpl {})
    }
}

#[tonic::async_trait]
impl greeter_server::Greeter for GreeterImpl {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
    async fn say_hello2(
        &self,
        request: Request<HelloRequest2>,
    ) -> Result<Response<HelloReply2>, Status> {
        let reply = HelloReply2 {
            message: format!("Hello2 {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

pub struct Greeter2Impl {}

impl Greeter2Impl {
    pub fn new_svc() -> greeter2_server::Greeter2Server<Self> {
        greeter2_server::Greeter2Server::new(Greeter2Impl {})
    }
}

#[tonic::async_trait]
impl greeter2_server::Greeter2 for Greeter2Impl {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let reply = HelloReply {
            message: format!("2Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
    async fn say_hello2(
        &self,
        request: Request<HelloRequest2>,
    ) -> Result<Response<HelloReply2>, Status> {
        let request = request.into_inner();
        let res = format!(
            "name:{},field1:{:?},field2:{:?},field3:{:?}",
            request.name,
            request.field1,
            request.field2,
            EnumOk::try_from(request.field3).unwrap()
        );
        let reply = HelloReply2 {
            message: format!("2Hello2 {}!", res),
        };
        Ok(Response::new(reply))
    }

    async fn say_hello3(
        &self,
        _request: Request<google::protobuf::Empty>,
    ) -> Result<Response<HelloReply>, Status> {
        let reply = HelloReply {
            message: "2Hello3 Empty!".to_string(),
        };
        Ok(Response::new(reply))
    }
}
