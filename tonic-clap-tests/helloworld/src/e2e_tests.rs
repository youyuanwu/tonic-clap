use std::{net::SocketAddr, time::Duration};

use tokio_util::sync::CancellationToken;
use tonic::{Request, Response, Status};

use crate::google::protobuf;

use super::helloworld::*;

struct Greeter {}

#[tonic::async_trait]
impl greeter_server::Greeter for Greeter {
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

struct Greeter2 {}

#[tonic::async_trait]
impl greeter2_server::Greeter2 for Greeter2 {
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
        _request: Request<protobuf::Empty>,
    ) -> Result<Response<HelloReply>, Status> {
        let reply = HelloReply {
            message: "2Hello3 Empty!".to_string(),
        };
        Ok(Response::new(reply))
    }
}

// creates a listener on a random port from os, and return the addr.
pub async fn create_listener_server() -> (tokio::net::TcpListener, std::net::SocketAddr) {
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    (listener, local_addr)
}

async fn run_server_block(listener: tokio::net::TcpListener, token: CancellationToken) {
    let greeter = Greeter {};
    let greeter2 = Greeter2 {};
    tonic::transport::Server::builder()
        .add_service(greeter_server::GreeterServer::new(greeter))
        .add_service(greeter2_server::Greeter2Server::new(greeter2))
        .serve_with_incoming_shutdown(
            tonic::transport::server::TcpIncoming::from(listener),
            async move { token.cancelled().await },
        )
        .await
        .unwrap();
}

// run the hand written cli
async fn run_client_manual(addr: SocketAddr, more_args: &[&str]) {
    run_client(addr, more_args, "hwcli").await
}

// run the generated cli
async fn run_client_gen(addr: SocketAddr, more_args: &[&str]) {
    run_client(addr, more_args, "hwgencli").await
}

async fn run_client(addr: SocketAddr, more_args: &[&str], bin: &str) {
    use std::process::Stdio;
    use tokio::process::Command;
    let shared_args = ["run", "--quiet", "--bin", bin, "--", "--url"];
    let mut child = Command::new("cargo")
        .current_dir("../") // workspace dir.
        .args(shared_args)
        .arg(format!("http://{addr}"))
        .args(more_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("fail to spawn");
    let ec = child.wait().await.unwrap();
    assert!(ec.success());
}

#[tokio::test]
async fn server_test() {
    let (l, addr) = create_listener_server().await;
    let token = CancellationToken::new();
    let svh = {
        let token = token.clone();
        tokio::spawn(async move { run_server_block(l, token).await })
    };

    tokio::time::sleep(Duration::from_secs(1)).await;
    println!("running client");
    run_client_manual(addr, &["greeter", "say-hello", "--name", "n1"]).await;
    run_client_manual(addr, &["greeter", "say-hello2", "--name", "n2"]).await;
    run_client_manual(addr, &["greeter2", "say-hello", "--name", "2n1"]).await;
    run_client_manual(
        addr,
        &[
            "greeter2",
            "say-hello2",
            "--name",
            "2n2",
            "--fcount",
            "3",
            "--field2",
            "v1",
            "--field2",
            "v2",
        ],
    )
    .await;
    run_client_manual(
        addr,
        &[
            "--json-data",
            r#"{ "name": "json_name" }"#,
            "greeter",
            "say-hello",
        ],
    )
    .await;

    run_client_gen(
        addr,
        &[
            "--json-data",
            r#"{ "name": "json_name_gen" }"#,
            "greeter",
            "say-hello",
        ],
    )
    .await;
    run_client_gen(
        addr,
        &[
            "--json-data",
            r#"{ "name": "n", "field2": [], "field3": 1 }"#, // enum is number in serde
            "greeter2",
            "say-hello2",
        ],
    )
    .await;

    run_client_gen(
        addr,
        &[
            "--json-data",
            r#"{}"#, // Empty data
            "greeter2",
            "say-hello3",
        ],
    )
    .await;

    run_client_gen(
        addr,
        &[
            "greeter2",
            "say-hello2",
            "--name",
            "g2s2",
            "--field1.fname",
            "fname",
            "--field1.fcount",
            "3",
            "--field2",
            "f2",
            "--field3",
            "1",
        ],
    )
    .await;

    token.cancel();
    svh.await.expect("task panic");
}
