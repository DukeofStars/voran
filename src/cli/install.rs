use std::{fs, process};

use clap::Args;
use reqwest::Client;
use voran::{
    jellyfish_install,
    package::PackageType,
    packages::{self, Packages},
    proj_dirs,
};

pub async fn install(args: InstallArgs) {
    // Make sure the package exists
    let package = packages::get_packages()
        .lazy()
        .await
        .unwrap()
        .get_package(&args.package)
        .expect("This package does not exist")
        .version(&args.version.unwrap_or("LATEST".to_string()))
        .expect("This version does not exist")
        .package()
        .unwrap();

    // Download the file
    let proj_dirs = proj_dirs();
    let cache_dir = proj_dirs.cache_dir();
    if !cache_dir.exists() {
        fs::create_dir_all(cache_dir).unwrap();
    }
    let out_file = voran::download_file(
        &Client::new(),
        &package.install.url,
        cache_dir.join(format!("{}.jellyfish", package.name)),
    )
    .await
    .expect("Failed to download file");
    println!("Download complete!, Installing...");

    match package.install.type_ {
        PackageType::Executable => {
            process::Command::new(out_file)
                .spawn()
                .expect("Failed to execute");
        }
        PackageType::JellyFish | PackageType::Wharf => {
            let installer = jellyfish_install::BasicJellyFishInstaller::new(out_file);
            jellyfish_install::install_to(&installer,
                            proj_dirs.data_dir().join("packages").join(&package.name),
                            proj_dirs.data_dir().join("bin"),
                            true,
                        )
                        .expect(
                            "Failed to install package. This may be caused by a corrupted package or a lack of sufficient privileges",
                        );

            // Store package information with package for later use.
            fs::write(
                proj_dirs
                    .data_dir()
                    .join("packages")
                    .join(&package.name)
                    .join("package.toml"),
                toml::to_string(&package).unwrap(),
            )
            .expect("Failed to store package information");

            if let PackageType::Wharf = package.install.type_ {
                wharf::run(
                    proj_dirs
                        .data_dir()
                        .join("packages")
                        .join(&package.name)
                        .join("build.rope"),
                );
            }
        }
    };

    println!(
        "Successfully installed {} v{}",
        package.friendly_name, package.version
    );
}

#[derive(Args)]
pub struct InstallArgs {
    /// Name of the package to be installed
    package: String,
    /// Optional version of the package
    #[clap(short, long)]
    version: Option<String>,
}
