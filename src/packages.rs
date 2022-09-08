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
    pub fn git(self) -> GitRepository {
        GitRepository::new(self.dir)
    }

    pub fn lazy(self) -> LazyPackages {
        LazyPackages { dir: self.dir }
    }

    pub fn load() -> LoadPackages {
        unimplemented!()
    }
}

pub trait Packages<GP: GetPackage> {
    fn get_package(self, name: &str) -> Option<GP>;
}

pub struct LazyPackages {
    dir: PathBuf,
}

impl Packages<LazyGetPackage> for LazyPackages {
    fn get_package(self, name: &str) -> Option<LazyGetPackage> {
        let path = self.dir.join(name);
        if !path.exists() {
            return None;
        }
        Some(LazyGetPackage { dir: path })
    }
}

// TODO
pub struct LoadPackages {}

impl Packages<LoadGetPackage> for LoadPackages {
    fn get_package(self, _name: &str) -> Option<LoadGetPackage> {
        todo!()
    }
}

pub trait GetPackage {
    fn version(&mut self, version: &str) -> Option<&mut Self>;

    fn package(&self) -> Option<Package>;
}

pub struct LazyGetPackage {
    dir: PathBuf,
}

impl GetPackage for LazyGetPackage {
    fn version(&mut self, version: &str) -> Option<&mut Self> {
        let path = self.dir.join(version);
        if !path.exists() {
            return None;
        }
        self.dir = path;
        Some(self)
    }

    fn package(&self) -> Option<Package> {
        let path = self.dir.join("package.toml");
        if !path.exists() {
            return None;
        }
        Some(toml::from_str(fs::read_to_string(path).unwrap().as_str()).unwrap())
    }
}

// TODO
pub struct LoadGetPackage {}

impl GetPackage for LoadGetPackage {
    fn version(&mut self, _version: &str) -> Option<&mut LoadGetPackage> {
        todo!()
    }

    fn package(&self) -> Option<Package> {
        todo!()
    }
}
