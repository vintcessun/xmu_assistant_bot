use crate::abi::router::handler::Router;
use anyhow::Result;
use std::path::Path;

const LOG_PATH: &str = "logs";

mod abi;
mod config;
mod logger;
mod logic;

#[tokio::main]
async fn main() -> Result<()> {
    if !Path::new(LOG_PATH).is_dir() {
        std::fs::create_dir(LOG_PATH)?;
        println!("不存在日志目录，已创建: {LOG_PATH}",);
    }

    let _guard = logger::init_logger(LOG_PATH, "trace");

    let mut router = abi::run(config::get_napcat_config()).await.unwrap();

    router.run().await;

    Ok(())
}
