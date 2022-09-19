use clap::Args;
use indicatif::ProgressBar;

use indicatif::ProgressStyle;

use voran::packages::GetPackage;
use voran::update;

pub async fn list(args: ListArgs) {
    // List upgradeable packages
    if args.upgradeable {
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner} [{bar:40.cyan/blue}] Checking for new application versions...")
                .unwrap()
                .progress_chars("#>-"),
        );
        let updates = update::check_for_updates(&pb)
            .await
            .expect("Failed to check for updates");
        pb.finish_and_clear();

        if updates.is_empty() {
            println!("Woohoo! No packages to be upgraded!");
        } else {
            println!("|{:30}|{:30}|{:15}|", "Name", "Id", "Version");
            println!(
                "|{:30}|{:30}|{:15}|",
                "-".repeat(30),
                "-".repeat(30),
                "-".repeat(15)
            );
            for update in updates {
                let get_package = GetPackage {
                    dir: update.path_old,
                };
                let new_get_package = GetPackage {
                    dir: update.path_new,
                };
                let package = get_package.package().expect("Invalid package");
                let new_package = new_get_package.package().expect("Invalid package");
                println!(
                    "|{:30}|{:30}|{:15}|",
                    package.friendly_name,
                    package.name,
                    format!("{} -> {}", package.version, new_package.version)
                );
            }
            println!(
                "|{:30}|{:30}|{:15}|",
                "-".repeat(30),
                "-".repeat(30),
                "-".repeat(15)
            );
        }
    }
    // List local packages
    else if args.local {
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
    else if args.remote || (!args.local && !args.remote && !args.upgradeable) {
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

#[derive(Args)]
pub struct ListArgs {
    #[clap(short, long)]
    pub remote: bool,
    #[clap(short, long)]
    pub local: bool,
    #[clap(short, long)]
    pub upgradeable: bool,
}
