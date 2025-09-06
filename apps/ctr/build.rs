fn main() {
    let containerd_dir = "../../protos/containerd";
    // TODO: some import protos are missing.
    let base_dir = format!("{containerd_dir}/github.com/containerd/containerd/api/services");
    // list all proto files
    let file_names = [
        "containers/v1/containers.proto",
        "content/v1/content.proto",
        "diff/v1/diff.proto",
        "events/v1/events.proto",
        "images/v1/images.proto",
        "introspection/v1/introspection.proto",
        "leases/v1/leases.proto",
        "namespaces/v1/namespace.proto",
        "sandbox/v1/sandbox.proto",
        "snapshots/v1/snapshots.proto",
        "tasks/v1/tasks.proto",
        "transfer/v1/transfer.proto",
        "version/v1/version.proto",
    ];
    let protos = file_names
        .iter()
        .map(|f| format!("{}/{}", base_dir, f))
        .collect::<Vec<_>>();
    tonic_clap_build::configure()
        .compile(&protos, &[containerd_dir])
        .unwrap();
}
