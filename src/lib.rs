use std::fs;

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

mod download;
pub use download::*;
pub mod jellyfish_install;
pub mod package;
pub mod packages;

pub fn load_local_config() -> Config {
    let proj_dirs = proj_dirs();
    let file = proj_dirs.config_dir().join("config.toml");

    if !file.exists() {
        println!("Error: configuration does not exist");
        println!("file: {file:?}");
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(&file, toml::to_string_pretty(&Config::default()).unwrap()).unwrap();
    }

    let config: Config = toml::from_str(
        fs::read_to_string(file)
            .expect("Config file does not exist")
            .as_str(),
    )
    .unwrap();
    config
}

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub git_repo_urls: Vec<(String, String)>,
}

pub fn proj_dirs() -> ProjectDirs {
    ProjectDirs::from("", "", "Voran").unwrap()
}
