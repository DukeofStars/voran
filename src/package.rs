use serde::{Deserialize, Serialize};

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
