use std::{env, fs::OpenOptions};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub servers: Vec<Server>,
    pub bridge_channel: u64,
}

pub fn load_config() -> Config {
    let path = env::current_dir().unwrap();
    let config_path = &(path.to_str().unwrap().to_owned() + "/config.ron");
    return ron::de::from_reader(OpenOptions::new().read(true).open(config_path).unwrap())
        .expect("Malformed config file");
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub container_name: String,
    pub display_name: String,
    // pub ip: String,
    // pub port: i32,
}
