use clap::{Args, Subcommand};

use voran::{save_config, Config};

pub async fn remote(config: &mut Config, args: RemoteArgs) {
    let subcommand = args.subcommand.unwrap_or(RemoteCommand::List);
    match subcommand {
        RemoteCommand::Add(args) => {
            if config.git_repo_urls.iter().any(|f| f.0 == args.name) {
                println!(
                    "Error: This remote already exists, remove it with `voran remote remove {}`",
                    args.name
                );
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
            for (name, url) in &config.git_repo_urls {
                println!("|{:30}|{:50}|", name, url);
            }
            println!("|{:30}|{:50}|", "-".repeat(30), "-".repeat(50));
        }
    }
}

#[derive(Args)]
pub struct RemoteArgs {
    #[clap(subcommand)]
    pub subcommand: Option<RemoteCommand>,
}

#[derive(Subcommand)]
pub enum RemoteCommand {
    /// Add a remote
    Add(RemoteAddArgs),
    /// Remove a remote
    Remove(RemoteRemoveArgs),
    List,
}

#[derive(Args)]
pub struct RemoteAddArgs {
    /// Name of the remote to be added (can be anything).
    pub name: String,
    /// Url of the remote
    pub url: String,
}

#[derive(Args)]
pub struct RemoteRemoveArgs {
    /// Name of the remote to be removed.
    pub name: String,
}
