use anyhow::Result;
use serde::Deserialize;
use serde_yaml::from_reader;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub db_url: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // read from /etc/config/app.yml, or ./app.yml, or from env CHAT_CONFIG

        let config = match (
            fs::File::open("/etc/config/app.yml"),
            fs::File::open("app.yml"),
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
