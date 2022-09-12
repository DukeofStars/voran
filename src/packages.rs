use std::{
    fs,
    path::{Path, PathBuf},
};

use git_rs::GitRepository;

use crate::{package::Package, proj_dirs};

/// Create a GetPackages at the package repository root.
pub fn get_packages() -> GetPackages {
    GetPackages {
        dir: proj_dirs().data_local_dir().to_path_buf(),
    }
}

/// Create a GetPackages at the package binaries root.
pub fn installed_packages() -> GetPackages {
    GetPackages {
        dir: proj_dirs().data_dir().to_path_buf().join("packages"),
    }
}

/// Helper class to get packages from repository.
pub struct GetPackages {
    dir: PathBuf,
}

impl GetPackages {
    /// Return the target repository as a GitRepository.
    pub async fn git(self) -> Result<GitRepository, failure::Error> {
        Ok(GitRepository::new(self.dir))
    }

    /// Lazy load the packages (on command).
    pub async fn lazy(self) -> Result<LazyPackages, failure::Error> {
        Ok(LazyPackages { dir: self.dir })
    }

    /// Asynchronously load the packages immediately.
    pub async fn load(self) -> Result<LoadPackages, failure::Error> {
        LoadPackages::begin(self.dir).await
    }

    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
        }
    }
}

/// A trait for something that can return packages (eg. LazyPackages, LoadPackages).
pub trait Packages {
    /// Get the desired package.
    fn get_package(&self, name: &str) -> Option<GetPackage>;
}

/// A Packages implementation that lazy loads the packages (ie. on command).
pub struct LazyPackages {
    pub dir: PathBuf,
}

impl Packages for LazyPackages {
    /// Get the desired package.
    fn get_package(&self, name: &str) -> Option<GetPackage> {
        let path = self.dir.join(name);
        if !path.exists() {
            return None;
        }
        Some(GetPackage { dir: path })
    }
}

pub struct LoadPackages {
    packages: Vec<PathBuf>,
    index: usize,
}

impl Packages for LoadPackages {
    /// Get the desired package.
    fn get_package(&self, name: &str) -> Option<GetPackage> {
        let path = self
            .packages
            .iter()
            .find(|x| x.file_name().unwrap_or_default().to_str().unwrap() == name && x.is_dir())?;
        Some(GetPackage {
            dir: path.to_path_buf(),
        })
    }
}

impl LoadPackages {
    /// Start loading packages
    async fn begin(dir: PathBuf) -> Result<LoadPackages, failure::Error> {
        let mut packages: Vec<PathBuf> = vec![];
        for entry in dir.read_dir()? {
            let entry = entry?;
            if !entry
                .file_name()
                .to_str()
                .unwrap()
                .to_string()
                .starts_with('.')
                && entry.file_type()?.is_dir()
            {
                packages.push(entry.path());
            }
        }
        Ok(Self { packages, index: 0 })
    }
}

impl Iterator for LoadPackages {
    type Item = GetPackage;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.packages.get(self.index)?.to_path_buf();
        let out = Some(GetPackage { dir: path });
        self.index += 1;
        out
    }
}

/// Helper class to get package information from folder
pub struct GetPackage {
    pub dir: PathBuf,
}

impl GetPackage {
    /// Navigate into the folder of a specific version
    pub fn version(&mut self, version: &str) -> Option<&mut Self> {
        let path = self.dir.join(version);
        if !path.exists() {
            return None;
        }
        self.dir = path;
        Some(self)
    }

    /// Load package.toml
    pub fn package(&self) -> Option<Package> {
        let path = self.dir.join("package.toml");
        if !path.exists() {
            return None;
        }
        Some(toml::from_str(fs::read_to_string(path).unwrap().as_str()).unwrap())
    }
}
