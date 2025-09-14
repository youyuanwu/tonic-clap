use clap::Args;

#[derive(Args, Debug)]
pub struct AutoCompleteArgs {
    /// Generate shell completion scripts for the specified shell
    #[arg(long, value_enum)]
    pub generate_completion: Option<clap_complete::Shell>,
}

pub(crate) fn get_current_binary_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .expect("no binary name")
}
