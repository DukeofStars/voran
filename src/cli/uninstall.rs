use std::fs;

use clap::Args;
use voran::{
    package::PackageType,
    packages::{self, Packages},
    proj_dirs,
};

pub async fn uninstall(args: UninstallArgs) {
    let package = packages::installed_packages()
        .lazy()
        .await
        .unwrap()
        .get_package(&args.package)
        .expect("This package does not exist")
        .package()
        .expect("Failed to get package.toml for this package");

    match package.install.type_ {
        PackageType::Executable => {
            println!("Fatal: This package cannot be uninstalled.")
        }
        PackageType::JellyFish | PackageType::Wharf => {
            fs::remove_dir_all(
                packages::installed_packages()
                    .lazy()
                    .await
                    .unwrap()
                    .get_package(&args.package)
                    .unwrap()
                    .dir,
            )
            .expect("Failed to remove files");

            if let PackageType::Wharf = package.install.type_ {
                wharf::reverse(
                    proj_dirs()
                        .data_dir()
                        .join("packages")
                        .join(&package.name)
                        .join("build.rope"),
                );
            }
        }
    }

    println!("Uninstallation successful");
}

#[derive(Args)]
pub struct UninstallArgs {
    /// Name of the package to be uninstalled.
    pub package: String,
}
