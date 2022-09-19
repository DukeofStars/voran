use std::{sync::Arc, thread, time::Duration};

use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::Mutex;
use voran::{packages, update, Config};

pub async fn update(config: &Config, args: UpdateArgs) {
    if !args.no_pull {
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
                pb.finish_and_clear();
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
                pb.finish_and_clear();
            });
            s.spawn(async {
                for (i, (name, url)) in config.git_repo_urls.iter().enumerate() {
                    repository.add_remote(&name, &url);

                    repository.pull(&name);
                    *index.lock().await = i as u64;
                }
            });
        });
    }

    // Check for updates
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} [{bar:40.cyan/blue}] Checking for new application versions...")
            .unwrap()
            .progress_chars("#>-"),
    );
    let updates = update::check_for_updates(&pb)
        .await
        .expect("Failed to check for package updates");
    pb.finish();

    println!("{} packages can be upgraded", updates.len());

    println!("Update successful");
}

#[derive(Args)]
pub struct UpdateArgs {
    /// Check for package updates without pulling latest changes from remotes.
    #[clap(long = "no-pull")]
    pub no_pull: bool,
}
