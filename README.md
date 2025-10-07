# tonic-clap
![ci](https://github.com/youyuanwu/tonic-clap/actions/workflows/CI.yml/badge.svg)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://raw.githubusercontent.com/youyuanwu/tonic-clap/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/tonic-clap)](https://crates.io/crates/tonic-clap)
[![Documentation](https://docs.rs/tonic-clap/badge.svg)](https://docs.rs/tonic-clap)

Auto generate [tonic](https://github.com/hyperium/tonic) and [clap](https://github.com/clap-rs/clap) gRPC CLI (commandline) tool from proto definition.

Features:
* Each grpc service is a cli verb
* Each grpc method is a cli verb under the service verb
* Each field in the proto Request is a cli option
* Nested field is a cli option with a path joined by "." 

Option construction relies on [bevy-reflect](https://github.com/bevyengine/bevy/tree/main/crates/bevy_reflect) dynamic reflection. Proc macro does not quite work, and prost code gen is much more difficult to write.

This is experimental is only suitable for testing or debugging you app.

# Get started
Add dependency:
```toml
[dependencies]
serde_json = "*"
tonic-clap = "*"
bevy_reflect = "*"

[build-dependencies]
tonic-clap-build = "*"
```
Add to your build.rs:
```rs
let mut builder = tonic_clap_build::configure().with_tonic_server(false);
let proto_file = Path::new("../../protos/helloworld.proto");
let proto_dir = proto_file.parent().unwrap();
builder.compile(&[proto_file], &[proto_dir]).unwrap();
```
Include generated file in your app:
```rs
pub mod helloworld {
    tonic::include_proto!("helloworld");
}
```
Add generated subcommand to your clap:
```rs
#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    command: helloworld::cli::CommandServices,
}
``` 

# Example
See example:
[proto](protos/helloworld.proto)
[generated-cli-app](./apps/hwgencli/)

See the clap help of the example app
```txt
cargo run --bin hwgencli -q -- greeter say-hello2 --name hi --help
Usage: hwgencli.exe greeter say-hello2 [OPTIONS]

Options:
      --name <NAME>             Arg: String
      --field1.fname <FNAME>    Arg: String
      --field1.fcount <FCOUNT>  Arg: i32
      --field2 <FIELD2>         Arg: Vec<String>
      --field3 <FIELD3>         Arg: i32
  -h, --help                    Print help
```

# Liscense
This project is licensed under the MIT license.