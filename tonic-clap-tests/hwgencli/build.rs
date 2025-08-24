use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = std::path::Path::new(&manifest_dir).join("gen");
    if !path.exists() {
        std::fs::create_dir_all(&path).unwrap();
    }

    println!("out {}", path.display());

    let mut builder = tonic_clap_build::configure().with_tonic_server(false);
    builder.get_cfg().out_dir(&path);

    let proto_file = Path::new("../../protos/helloworld.proto");
    let proto_dir = proto_file.parent().unwrap();
    assert!(proto_file.exists());
    assert!(proto_dir.exists());
    assert!(path.exists());
    builder.compile(&[proto_file], &[proto_dir]).unwrap();
    Ok(())
}
