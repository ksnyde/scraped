use color_eyre::Result;
use dotenv::dotenv;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum LogLevel {
    Debug(),
    Info(),
    Warn(),
    Error(),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub log_level: LogLevel,
}

impl Config {
    pub fn new() -> Config {
        Config {
            log_level: LogLevel::Debug(),
        }
    }

    pub fn from_env() -> Result<Config> {
        dotenv().ok();
        let c = Config::new();
        // c.merge(config::Environment::default())?;

        // c.try_into()
        //     .context("loading configuration from environment");

        Ok(c)
    }
}
