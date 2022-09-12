use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::{fs, process, sync::Arc, thread, time::Duration};
use tokio::sync::Mutex;

use voran::{package::PackageType, packages::Packages, *};

#[tokio::main]
async fn main() {
    let mut config = load_local_config();
    let cli = Cli::parse();

    match cli.subcommand {
        Command::Update => {
            let mut repository = packages::get_packages().git().await.unwrap();

            let remotes = repository.remotes();
            let remotes_len = remotes.len();

            let index: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));

            let pb = ProgressBar::new(remotes_len.saturating_sub(1) as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner} [{bar:40.cyan/blue}] Setting up remotes...")
                    .unwrap()
                    .progress_chars("#>-"),
            );

            tokio_scoped::scope(|s| {
                s.spawn(async {
                    while index.lock().await.saturating_add(0) < pb.length().unwrap() {
                        pb.set_position(index.lock().await.saturating_add(0));
                        thread::sleep(Duration::from_millis(100));
                    }
                    pb.finish();
                });
                s.spawn(async {
                    for (i, remote) in remotes.iter().enumerate() {
                        repository.remove_remote(&remote);
                        *index.lock().await = i as u64;
                    }
                });
            });

            let pb = ProgressBar::new(config.git_repo_urls.len().saturating_sub(1) as u64);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner} [{bar:40.cyan/blue}] Pulling remotes...")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            let index: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
            tokio_scoped::scope(|s| {
                s.spawn(async {
                    while index.lock().await.saturating_add(0) < pb.length().unwrap() {
                        pb.set_position(index.lock().await.saturating_add(0));
                        thread::sleep(Duration::from_millis(100));
                    }
                    pb.finish();
                });
                s.spawn(async {
                    for (i, (name, url)) in config.git_repo_urls.iter().enumerate() {
                        repository.add_remote(&name, &url);

                        repository.pull(&name);
                        *index.lock().await = i as u64;
                    }
                });
            });
            println!("Update successful");
        }
        Command::Install(args) => {
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
        Command::Uninstall(args) => {
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
        Command::List(args) => {
            // List local packages
            if args.local {
                let packages = voran::packages::installed_packages()
                    .load()
                    .await
                    .expect("Failed to load packages");
                println!("|{:30}|{:30}|{:10}|", "Name", "Id", "Version");
                println!(
                    "|{:30}|{:30}|{:10}|",
                    "-".repeat(30),
                    "-".repeat(30),
                    "-".repeat(10)
                );
                for package in packages {
                    let package = package.package().unwrap();
                    println!(
                        "|{:30}|{:30}|{:10}|",
                        package.friendly_name, package.name, package.version
                    );
                }
                println!(
                    "|{:30}|{:30}|{:10}|",
                    "-".repeat(30),
                    "-".repeat(30),
                    "-".repeat(10)
                );
            }
            // List remote packages
            else if args.remote || (!args.local && !args.remote) {
                let packages = voran::packages::get_packages()
                    .load()
                    .await
                    .expect("Failed to load packages");
                println!("|{:30}|{:30}|{:10}|", "Name", "Id", "Version");
                println!(
                    "|{:30}|{:30}|{:10}|",
                    "-".repeat(30),
                    "-".repeat(30),
                    "-".repeat(10)
                );
                for mut package in packages {
                    let package = package
                        .version("LATEST")
                        .expect("Failed to load package")
                        .package()
                        .unwrap();
                    println!(
                        "|{:30}|{:30}|{:10}|",
                        package.friendly_name, package.name, package.version
                    );
                }
                println!(
                    "|{:30}|{:30}|{:10}|",
                    "-".repeat(30),
                    "-".repeat(30),
                    "-".repeat(10)
                );
            }
        }
        Command::Remote(args) => {
            let subcommand = args.subcommand.unwrap_or(RemoteCommand::List);
            match subcommand {
                RemoteCommand::Add(args) => {
                    if config.git_repo_urls.iter().any(|f| f.0 == args.name) {
                        println!("Error: This remote already exists, remove it with `voran remote remove {}`", args.name);
                    }
                    config.git_repo_urls.push((args.name, args.url));

                    save_config(&config).expect("Failed to save configuration");
                }
                RemoteCommand::Remove(args) => {
                    let config: Vec<(String, String)> = config
                        .git_repo_urls
                        .iter()
                        .filter(|x| x.0 != args.name)
                        .cloned()
                        .collect();

                    save_config(&Config {
                        git_repo_urls: config,
                    })
                    .expect("Failed to save configuration");
                }
                RemoteCommand::List => {
                    println!("|{:30}|{:50}|", "Name", "Url");
                    println!("|{:30}|{:50}|", "-".repeat(30), "-".repeat(50));
                    for (name, url) in config.git_repo_urls {
                        println!("|{:30}|{:50}|", name, url);
                    }
                    println!("|{:30}|{:50}|", "-".repeat(30), "-".repeat(50));
                }
            }
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
    /// Uninstall a package
    Uninstall(UninstallArgs),
    /// List all packages from a remote
    List(ListArgs),
    /// Manage remotes
    Remote(RemoteArgs),
}

#[derive(Args)]
struct RemoteArgs {
    #[clap(subcommand)]
    subcommand: Option<RemoteCommand>,
}

#[derive(Subcommand)]
enum RemoteCommand {
    /// Add a remote
    Add(RemoteAddArgs),
    /// Remove a remote
    Remove(RemoteRemoveArgs),
    List,
}

#[derive(Args)]
struct RemoteAddArgs {
    /// Name of the remote to be added (can be anything).
    name: String,
    /// Url of the remote
    url: String,
}

#[derive(Args)]
struct RemoteRemoveArgs {
    /// Name of the remote to be removed.
    name: String,
}

#[derive(Args)]
struct ListArgs {
    #[clap(short, long)]
    remote: bool,
    #[clap(short, long)]
    local: bool,
}

#[derive(Args)]
struct InstallArgs {
    /// Name of the package to be installed
    package: String,
    /// Optional version of the package
    #[clap(short, long)]
    version: Option<String>,
}

#[derive(Args)]
struct UninstallArgs {
    /// Name of the package to be uninstalled.
    package: String,
}
