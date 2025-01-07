use dirs::config_dir;
use fs;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::io::{Read, Write};
use std::path::PathBuf;
use toml;

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub ip: String,
    pub data_dir: String,
    pub user: User,
}

#[derive(Deserialize, Serialize)]
pub struct User {
    pub name: String,
    pub pass: String,
}

pub fn get_config() -> Result<Config, Box<dyn Error>> {
    let path = get_config_path()?.join("config.toml");

    let mut toml = fs::load_file(&path)?;
    let mut buf: Vec<u8> = Vec::new();
    toml.read_to_end(&mut buf)?;
    let toml = String::from_utf8_lossy(&buf);
    let config: Config = toml::from_str(&toml)?;
    Ok(config)
}

pub fn crete_conf() -> Result<(), Box<dyn Error>> {
    let path = get_config_path()?;

    fs::create_dir(&path)?;
    let config: Config = Config {
        ip: "127.0.0.1:8080".to_string(),
        data_dir: "/home/zeroone/nimbus_server/".to_string(),
        user: User {
            name: "Alice".to_string(),
            pass: "AliceTheBest123".to_string(),
        },
    };

    let toml = toml::to_string_pretty(&config)?;
    let path = path.join("config.toml");
    let mut file = fs::create_file(&path)?;
    file.write_all(toml.as_bytes())?;
    Ok(())
}

pub fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    let config_dir = config_dir().unwrap().join("nimbus_server/");
    Ok(config_dir)
}
