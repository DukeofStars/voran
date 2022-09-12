use serde::{Deserialize, Serialize};

/// Package information
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Package {
    pub name: String,
    pub friendly_name: String,
    pub version: String,
    pub install: InstallInfo,
}

/// Package install information
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct InstallInfo {
    pub url: String,
    pub type_: PackageType,
}

/// Package install type
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum PackageType {
    Executable,
    JellyFish,
    Wharf,
}
