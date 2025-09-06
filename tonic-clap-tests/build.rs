fn main() {
    tonic_clap_build::compile_protos(&["../protos/helloworld.proto"]).unwrap();
}
