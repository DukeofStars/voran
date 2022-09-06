use std::{fs, path::PathBuf, process};

use clap::{Args, Parser, Subcommand};

use indicatif::{ProgressBar, ProgressStyle};

use voran::*;

#[tokio::main]
async fn main() {
    let config = load_local_config();
    let cli = Cli::parse();

    match cli.subcommand {
        Command::Update => {
            let proj_dirs = proj_dirs();

            let local_repo_path = proj_dirs.data_local_dir();
            let mut repository = git_rs::GitRepository::new(PathBuf::from(local_repo_path));

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
            let path = proj_dirs().data_local_dir().join(args.package);
            if path.is_file() || !path.exists() {
                panic!("This package does not exist")
            }

            // Make sure the version exists
            let version = args.version.unwrap_or("LATEST".to_string());
            let version_path = path.join(version);
            if version_path.is_file() || !version_path.exists() {
                panic!("This version does not exist for this package");
            }

            // Load header
            let package_file = version_path.join("package.toml");
            if !package_file.exists() || !package_file.is_file() {
                panic!("Invalid package version");
            }
            let package: Package =
                toml::from_str(fs::read_to_string(package_file).unwrap().as_str()).unwrap();

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

fn load_local_config() -> Config {
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
