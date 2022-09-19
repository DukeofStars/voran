use std::fs;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

mod _lib;
pub use _lib::*;

/// Load config saved on local disk.
pub fn load_local_config() -> Config {
    let proj_dirs = proj_dirs();
    let file = proj_dirs.config_dir().join("config.toml");

    if !file.exists() {
        println!("Error: configuration does not exist");
        println!("file: {file:?}");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, toml::to_string_pretty(&Config::default()).unwrap()).unwrap();
        std::process::exit(1);
    }

    let config: Config = toml::from_str(
        fs::read_to_string(file)
            .expect("Config file does not exist")
            .as_str(),
    )
    .unwrap();
    config
}

/// Save config to local disk
pub fn save_config(config: &Config) -> Result<(), failure::Error> {
    let proj_dirs = proj_dirs();
    let file = proj_dirs.config_dir().join("config.toml");

    fs::write(&file, toml::to_string(config)?)?;

    Ok(())
}

/// Serialize and Deserializeable Configuration struct.
#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub git_repo_urls: Vec<(String, String)>,
}

/// Get directories::ProjectDirs of this application.
pub fn proj_dirs() -> ProjectDirs {
    ProjectDirs::from("", "", "Voran").unwrap()
}
