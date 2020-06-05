use serde::Deserialize;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;

fn get_config_file_path() -> Result<String, String> {
    let xdg_home = env::var("XDG_CONFIG_HOME");
    let folder = match xdg_home {
        Err(_) => {
            let home = env::var("HOME").map_err(|_| "HOME variable not set".to_string())?;
            Path::new(&home).join(".config/anthill/config.toml")
        }
        Ok(xdg_home) => Path::new(&xdg_home).join("anthill/config.toml"),
    };

    folder
        .into_os_string()
        .into_string()
        .map_err(|_| "could not find config file".to_string())
}

#[derive(Deserialize, Debug)]
pub struct Mb {
    pub local: String,
    pub remote: String
}

#[derive(Deserialize,Debug)]
pub struct Config {
    pub url: String,
    pub port: u16,
    pub user: String,
    pub pass_cmd: String,
    pub with_tls: bool,
    pub folder: String,
    pub mailboxes: HashMap<String, Mb>,
}

pub fn get_config() -> Result<HashMap<String, Config>, String> {
    let config_folder = get_config_file_path()?;
    let content =
        fs::read(&config_folder).map_err(|_| format!("could not read {}", config_folder))?;
    let config_str = std::str::from_utf8(&content).map_err(|_| "config file is not valid utf-8")?;

    let config: HashMap<String, Config> =
        toml::from_str(config_str).map_err(|e| format!("could not parse config file: {}", e))?;

    Ok(config)
}
