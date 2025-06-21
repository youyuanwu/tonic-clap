use std::{io, path::Path};

use code_gen::ServiceGenerator;
use tonic_build::Config;

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
        let g1 = tonic_build::configure()
            .build_server(self.tonic_server)
            .service_generator();
        let g2 = self.service_generator();
        let g = MultiGen::new(g1, g2);
        self.cfg.service_generator(Box::new(g));
        // add clap attr
        self.cfg
            .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
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
pub fn compile_protos(proto: impl AsRef<Path>) -> io::Result<()> {
    let proto_path: &Path = proto.as_ref();

    // directory the main .proto file resides in
    let proto_dir = proto_path
        .parent()
        .expect("proto file should reside in a directory");

    self::configure().compile(&[proto_path], &[proto_dir])?;

    Ok(())
}
