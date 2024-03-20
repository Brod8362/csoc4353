use std::{error::Error, fs};

use rocket::figment::providers::{Format, Toml};
use serde::Deserialize;

fn default_database_uri() -> String {
    "sqlite://database.db".to_owned()
}

fn default_max_connections() -> u32 {
    1
}

fn default_jwt_secret() -> String {
    "debug".to_owned()
}

#[derive(Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_database_uri")]
    pub database_uri: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String
}

impl AppConfig {
    pub fn secret_as_bytes(&self) -> Box<&[u8]> {
        Box::new(self.jwt_secret.as_bytes())
    }
}

pub fn load_config(path: String) -> Result<AppConfig, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let c: AppConfig = Toml::from_str(&content)?;
    Ok(c)   
}