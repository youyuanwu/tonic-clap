pub mod helloworld {
    tonic::include_proto!("helloworld");
}
pub mod google {
    pub mod protobuf {
        tonic::include_proto!("google.protobuf");
    }
}

#[cfg(test)]
mod ref_tests;

#[cfg(test)]
mod e2e_tests;
