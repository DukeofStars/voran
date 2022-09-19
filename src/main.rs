use clap::{Parser, Subcommand};
use voran::load_local_config;

mod cli;

#[tokio::main]
async fn main() {
    let mut config = load_local_config();
    let cli = Cli::parse();

    match cli.subcommand {
        Command::Update(args) => {
            cli::update(&config, args).await;
        }
        Command::Install(args) => {
            cli::install(args).await;
        }
        Command::Uninstall(args) => {
            cli::uninstall(args).await;
        }
        Command::List(args) => {
            cli::list(args).await;
        }
        Command::Remote(args) => {
            cli::remote(&mut config, args).await;
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
    Update(cli::UpdateArgs),
    /// Install a package
    Install(cli::InstallArgs),
    /// Uninstall a package
    Uninstall(cli::UninstallArgs),
    /// List all packages from a remote
    List(cli::ListArgs),
    /// Manage remotes
    Remote(cli::RemoteArgs),
}
