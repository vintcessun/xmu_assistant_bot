use crate::abi::router::handler::Router;
use anyhow::Result;

const LOG_PATH: &str = "logs";

mod abi;
mod api;
pub mod config;
mod logger;
mod logic;

#[tokio::main]
async fn main() -> Result<()> {
    config::ensure_dir(LOG_PATH);
    config::ensure_dir(config::DATA_DIR);

    let _guard = logger::init_logger(LOG_PATH, "trace");

    let mut router = abi::run(config::get_napcat_config()).await.unwrap();

    router.run().await;

    Ok(())
}
