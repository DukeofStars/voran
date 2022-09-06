use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

mod download;
pub use download::*;
pub mod git;
pub mod jellyfish_install;

#[derive(Deserialize, Serialize, Default)]
pub struct Config {
    pub git_repo_urls: Vec<(String, String)>,
}

pub fn proj_dirs() -> ProjectDirs {
    ProjectDirs::from("", "", "Voran").unwrap()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub friendly_name: String,
    pub version: String,
    pub install: InstallInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallInfo {
    pub url: String,
    pub type_: PackageType,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum PackageType {
    Executable,
    JellyFish,
    Wharf,
}
