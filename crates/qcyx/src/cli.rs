#[tokio::main]
async fn main() {
    qcyx_i18n::localize();

    if let Err(e) = qcyx_cli::run().await {
        eprintln!("Error: {e}");

        std::process::exit(1);
    }
}
