use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub napcat: ServerConfig,
    pub bot: BotConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub access_token: Option<String>,
    pub reconnect_interval_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 3001,
            access_token: Some("xmu_assistant_bot".to_string()),
            reconnect_interval_secs: 10,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BotConfig {
    pub qq: u64,
    pub command_prefix: String,
    pub admin_qqs: Vec<u64>,
}

impl Default for BotConfig {
    fn default() -> Self {
        BotConfig {
            qq: 123456789,
            command_prefix: "/".to_string(),
            admin_qqs: vec![],
        }
    }
}

pub async fn load_config(path: &str) -> Result<Config> {
    let config_contents = tokio::fs::read_to_string(path).await?;
    let config: Config = toml::from_str(&config_contents)?;
    Ok(config)
}

pub async fn save_config(path: &str, config: &Config) -> Result<()> {
    let config_contents = toml::to_string_pretty(config)?;
    tokio::fs::write(path, config_contents).await?;
    Ok(())
}
