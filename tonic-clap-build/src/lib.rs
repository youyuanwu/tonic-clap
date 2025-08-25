use std::{io, path::Path};

use code_gen::ServiceGenerator;
use tonic_prost_build::Config;

use crate::multi_gen::MultiGen;

// mod client;
mod code_gen;
// mod server;
mod multi_gen;

// code gen builder
pub struct Builder {
    cfg: Config,
    tonic_server: bool,
}

pub fn configure() -> Builder {
    Builder {
        cfg: Config::new(),
        tonic_server: true,
    }
}

impl Builder {
    /// Compile the .proto files and execute code generation.
    pub fn compile(
        mut self,
        protos: &[impl AsRef<Path>],
        includes: &[impl AsRef<Path>],
    ) -> io::Result<()> {
        // merge tonic gen and clap gen.
        let g1 = tonic_prost_build::configure()
            .build_server(self.tonic_server)
            .service_generator();
        let g2 = self.service_generator();
        let g = MultiGen::new(g1, g2);
        self.cfg.service_generator(Box::new(g));

        // default only works for message structs.
        self.cfg.message_attribute(".", "#[serde(default)]");
        self.cfg.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize, tonic_clap::TonicClap, bevy_reflect::Reflect)]");

        // disable recusive known types
        // See bevy issue: https://github.com/bevyengine/bevy/issues/8965
        for message in &[
            "google.protobuf.FileDescriptorProto",
            "google.protobuf.DescriptorProto",
        ] {
            self.cfg
                .type_attribute(message, "#[reflect(no_field_bounds)]");
        }

        self.cfg.disable_comments(["."]);
        // self.cfg
        //     .field_attribute(".", "#[arg(long, default_value = \"\")]");
        self.cfg.compile_well_known_types();
        self.cfg.compile_protos(protos, includes)?;
        Ok(())
    }

    pub fn get_cfg(&mut self) -> &mut Config {
        &mut self.cfg
    }

    pub fn with_tonic_server(mut self, server: bool) -> Self {
        self.tonic_server = server;
        self
    }

    // turn builder into generator
    pub fn service_generator(&self) -> Box<dyn prost_build::ServiceGenerator> {
        Box::new(ServiceGenerator::new())
    }
}

/// Public entrypoint to the build
pub fn compile_protos(protos: &[impl AsRef<Path>]) -> io::Result<()> {
    let proto_paths = protos;

    // directory the main .proto file resides in
    let proto_dirs = proto_paths
        .iter()
        .map(|proto_path| {
            proto_path
                .as_ref()
                .parent()
                .expect("proto file should reside in a directory")
        })
        .collect::<std::collections::HashSet<_>>();
    self::configure().compile(proto_paths, &proto_dirs.into_iter().collect::<Vec<_>>())?;

    Ok(())
}
