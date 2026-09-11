use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    qcyx_i18n::localize();
    qcyx_cli::run().await
}
