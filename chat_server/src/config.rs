use anyhow::Result;
use serde::Deserialize;
use serde_yaml::from_reader;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // read from /etc/config/app.yaml, or ./app.yaml, or from env CHAT_CONFIG

        let config = match (
            fs::File::open("/etc/config/app.yaml"),
            fs::File::open("app.yaml"),
            std::env::var("CHAT_CONFIG"),
        ) {
            (Ok(reader), _, _) => from_reader(reader),
            (_, Ok(reader), _) => from_reader(reader),
            (_, _, Ok(path)) => {
                let reader = fs::File::open(path)?;
                from_reader(reader)
            }
            _ => return Err(anyhow::anyhow!("Config file not found")),
        };
        Ok(config?)
    }
}
