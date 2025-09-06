use std::{net::SocketAddr, time::Duration};

use tokio_util::sync::CancellationToken;

use crate::server::{Greeter2Impl, GreeterImpl};

// creates a listener on a random port from os, and return the addr.
pub async fn create_listener_server() -> (tokio::net::TcpListener, std::net::SocketAddr) {
    let addr: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    (listener, local_addr)
}

async fn run_server_block(listener: tokio::net::TcpListener, token: CancellationToken) {
    let greeter = GreeterImpl::new_svc();
    let greeter2 = Greeter2Impl::new_svc();
    tonic::transport::Server::builder()
        .add_service(greeter)
        .add_service(greeter2)
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

const CARGO_ARGS: &[&str] = &["run", "--quiet", "--bin"];

async fn run_client(addr: SocketAddr, more_args: &[&str], bin: &str) {
    use std::process::Stdio;
    use tokio::process::Command;
    let url = &format!("http://{addr}");
    let mut cargo_args = Vec::from(CARGO_ARGS);
    cargo_args.extend_from_slice(&[bin, "--"]);
    let mut app_args = Vec::from(&["tcp", "--url", url]);
    app_args.extend_from_slice(more_args);

    // Run this with the in process parsing first
    if bin == "hwgencli" {
        use clap::Parser;
        let mut app_args2 = vec!["hwgencli"];
        app_args2.extend_from_slice(&app_args);
        crate::HWArgs::try_parse_from(app_args2).unwrap();
    }
    let mut child = Command::new("cargo")
        .current_dir("../") // workspace dir.
        .args(&cargo_args)
        .args(&app_args)
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
            "--opt_string",
            "optstring",
            "--one_of_field.OneOfInt",
            "123",
        ],
    )
    .await;

    token.cancel();
    svh.await.expect("task panic");
}

// ensures the ctr can show hwlp message
#[tokio::test]
async fn containerd_cli_compile_test() {
    let addr = "127.0.0.1:50051".parse().unwrap();
    let more_args = &["--help"];
    run_client(addr, more_args, "ctr").await
}
