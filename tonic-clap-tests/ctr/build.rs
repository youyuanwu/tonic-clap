fn main() {
    let protos = [
        "../../protos/containerd/containers.proto",
        "../../protos/containerd/content.proto",
    ];
    tonic_clap_build::compile_protos(&protos).unwrap();
}
