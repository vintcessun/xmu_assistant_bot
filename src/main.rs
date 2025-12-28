use anyhow::Result;
use std::path::Path;
use tracing::{debug, info, trace, warn};

use crate::abi::router::handler::Router;

const CONFIG_PATH: &str = "config.toml";
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

    let _guard = logger::init_logger(LOG_PATH, "info");

    if !Path::new(CONFIG_PATH).is_file() {
        let default_config = config::Config::default();
        config::save_config(CONFIG_PATH, &default_config).await?;
        warn!("不存在配置文件，已创建默认配置: {CONFIG_PATH}");
        trace!(?default_config);
        return Ok(());
    }

    let config = config::load_config(CONFIG_PATH).await.unwrap();
    info!("配置文件加载成功: {CONFIG_PATH}");
    debug!(?config);

    let mut router = abi::run(config).await.unwrap();

    router.run().await;

    Ok(())
}
