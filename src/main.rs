use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use std::{fs, process};

use voran::{
    package::PackageType,
    packages::{GetPackage, Packages},
    *,
};

#[tokio::main]
async fn main() {
    let config = load_local_config();
    let cli = Cli::parse();

    match cli.subcommand {
        Command::Update => {
            let mut repository = packages::get_packages().git();

            let remotes = repository.remotes();
            let pb = ProgressBar::new(remotes.len() as u64 - 1);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{bar:40.cyan/blue}]")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            for (index, remote) in remotes.iter().enumerate() {
                repository.remove_remote(&remote);
                pb.set_position(index as u64);
            }

            let pb = ProgressBar::new((config.git_repo_urls.len() - 1) as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("[{bar:40.cyan/blue}]")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            for (index, (name, url)) in config.git_repo_urls.iter().enumerate() {
                repository.add_remote(&name, &url);

                repository.pull(&name);
                pb.set_position(index as u64);
            }
            println!("Update successful");
        }
        Command::Install(args) => {
            // Make sure the package exists
            let package = packages::get_packages()
                .lazy()
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
            let out_file = voran::download(&package.install.url, cache_dir.to_path_buf())
                .await
                .expect("Failed to download file");
            println!("Download complete!, Installing...");

            match package.install.type_ {
                PackageType::Executable => {
                    process::Command::new(out_file)
                        .spawn()
                        .expect("Failed to execute");
                }
                PackageType::JellyFish => {
                    let installer = jellyfish_install::JellyFishInstaller::new(out_file);
                    installer
                        .install_to(
                            proj_dirs.data_dir().join("packages").join(package.name),
                            proj_dirs.data_dir().join("bin"),
                            true,
                        )
                        .expect(
                            "Failed to install package. This may be caused by a corrupted package or a lack of sufficient privileges",
                        );
                }
                PackageType::Wharf => todo!(),
            };

            println!(
                "Successfully installed {} v{}",
                package.friendly_name, package.version
            );
        }
    }
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Update the local repository
    Update,
    /// Install a package
    Install(InstallArgs),
}

#[derive(Args)]
struct InstallArgs {
    /// Name of the package to be installed
    package: String,
    /// Optional version of the package
    #[clap(short, long)]
    version: Option<String>,
}
