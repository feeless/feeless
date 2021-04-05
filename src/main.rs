#![forbid(unsafe_code)]
#![cfg_attr(feature = "deny_warnings", deny(warnings))]

use feeless::cli;
use tracing::error;

#[tokio::main]
async fn main() {
    let result = cli::run().await;
    if let Err(err) = result {
        error!("Exiting because of an error: {:?}", err);
        std::process::exit(1);
    }
}
