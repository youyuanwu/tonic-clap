fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_clap_build::compile_protos(&["../../protos/helloworld.proto"])?;
    Ok(())
}
