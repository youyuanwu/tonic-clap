use prost_build::{Service, ServiceGenerator};

/// Merge 2 generate into 1.
pub struct MultiGen {
    gen1: Box<dyn ServiceGenerator>,
    gen2: Box<dyn ServiceGenerator>,
}

impl MultiGen {
    pub fn new(gen1: Box<dyn ServiceGenerator>, gen2: Box<dyn ServiceGenerator>) -> Self {
        MultiGen { gen1, gen2 }
    }
}

impl ServiceGenerator for MultiGen {
    fn generate(&mut self, service: Service, buf: &mut String) {
        self.gen1.generate(service.clone(), buf);
        self.gen2.generate(service, buf);
    }

    fn finalize(&mut self, buf: &mut String) {
        self.gen1.finalize(buf);
        self.gen2.finalize(buf);
    }

    fn finalize_package(&mut self, package: &str, buf: &mut String) {
        self.gen1.finalize_package(package, buf);
        self.gen2.finalize_package(package, buf);
    }
}
