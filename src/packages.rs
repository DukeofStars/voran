use std::{fs, path::PathBuf};

use git_rs::GitRepository;

use crate::{package::Package, proj_dirs};

pub fn get_packages() -> GetPackages {
    GetPackages {
        dir: proj_dirs().data_local_dir().to_path_buf(),
    }
}

pub fn installed_packages() -> GetPackages {
    GetPackages {
        dir: proj_dirs().data_dir().to_path_buf().join("packages"),
    }
}

pub struct GetPackages {
    dir: PathBuf,
}

impl GetPackages {
    pub async fn git(self) -> Result<GitRepository, failure::Error> {
        Ok(GitRepository::new(self.dir))
    }

    pub async fn lazy(self) -> Result<LazyPackages, failure::Error> {
        Ok(LazyPackages { dir: self.dir })
    }

    pub async fn load(self) -> Result<LoadPackages, failure::Error> {
        LoadPackages::begin(self.dir).await
    }
}

pub trait Packages {
    fn get_package(self, name: &str) -> Option<GetPackage>;
}

pub struct LazyPackages {
    pub dir: PathBuf,
}

impl Packages for LazyPackages {
    fn get_package(self, name: &str) -> Option<GetPackage> {
        let path = self.dir.join(name);
        if !path.exists() {
            return None;
        }
        Some(GetPackage { dir: path })
    }
}

// TODO
pub struct LoadPackages {
    packages: Vec<PathBuf>,
    index: usize,
}

impl Packages for LoadPackages {
    fn get_package(self, _name: &str) -> Option<GetPackage> {
        todo!()
    }
}

impl LoadPackages {
    // Start loading packages
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

pub struct GetPackage {
    pub dir: PathBuf,
}

impl GetPackage {
    pub fn version(&mut self, version: &str) -> Option<&mut Self> {
        let path = self.dir.join(version);
        if !path.exists() {
            return None;
        }
        self.dir = path;
        Some(self)
    }

    pub fn package(&self) -> Option<Package> {
        let path = self.dir.join("package.toml");
        if !path.exists() {
            return None;
        }
        Some(toml::from_str(fs::read_to_string(path).unwrap().as_str()).unwrap())
    }
}
