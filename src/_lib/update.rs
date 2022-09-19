use std::{path::PathBuf, sync::Arc, thread, time::Duration};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use indicatif::ProgressBar;

use crate::packages::{self, Packages};

pub async fn check_for_updates(pb: &ProgressBar) -> Result<Vec<Update>, failure::Error> {
    let installed: Vec<_> = packages::installed_packages().load().await?.collect();
    pb.set_length(installed.len() as u64);

    let mut updates: Vec<Update> = vec![];

    let index: Arc<Mutex<u64>> = Arc::new(Mutex::new(0));
    tokio_scoped::scope(|s| -> Result<(), failure::Error> {
        s.spawn(async {
            while index.lock().await.saturating_sub(0) < pb.length().unwrap() {
                pb.set_position(index.lock().await.saturating_sub(0));
                thread::sleep(Duration::from_millis(100));
            }
        });
        s.spawn(async {
            for pkg in installed {
                let package = pkg
                    .package()
                    .ok_or(failure::err_msg("package.toml not found"))
                    .expect("Failed to check package");
                let tmp = packages::get_packages()
                    .lazy()
                    .await
                    .expect("Failed to check package");
                let mut tmp = tmp
                    .get_package(&package.name)
                    .ok_or(failure::err_msg("This package does not exist"))
                    .expect("Failed to check package");
                let new_pkg = tmp
                    .version("LATEST")
                    .ok_or(failure::err_msg(
                        "This package does not have a LATEST release",
                    ))
                    .expect("Failed to check package");
                let new_package = new_pkg
                    .package()
                    .ok_or(failure::err_msg("package.toml not found")) //?;
                    .expect("Failed to check package");

                if package.version != new_package.version {
                    updates.push(Update {
                        path_old: pkg.dir,
                        path_new: new_pkg.dir.to_owned(),
                    });
                }
                *index.lock().await += 1;
            }
        });
        Ok(())
    })?;

    Ok(updates)
}

#[derive(Serialize, Deserialize)]
pub struct Update {
    pub path_old: PathBuf,
    pub path_new: PathBuf,
}

impl Update {
    pub async fn apply(&self) -> Result<(), failure::Error> {
        self.reinstall().await?;
        Ok(())
    }

    pub async fn reinstall(&self) -> Result<(), failure::Error> {
        let _ = self.path_new;
        let _ = self.path_old;
        Ok(())
    }
}
