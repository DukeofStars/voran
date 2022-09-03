use std::{
    env, fs,
    path::PathBuf,
    process::{self, Command, Stdio},
};

pub struct GitRepository {
    path: PathBuf,
}

impl GitRepository {
    pub fn new(path: PathBuf) -> Self {
        if !path.exists() {
            fs::create_dir_all(&path).unwrap();
        }
        if path.is_file() {
            panic!(
                "Needs to be a directory, not a file, {}",
                &path.to_str().unwrap()
            );
        }
        let current_dir = env::current_dir().expect("Failed to get return directory");
        env::set_current_dir(&path).unwrap();

        let _status = Command::new("git")
            .args(["init", "--quiet"])
            .status()
            .expect("Failed to spawn process");

        env::set_current_dir(current_dir).unwrap();

        Self { path }
    }

    pub fn remotes(&self) -> Vec<String> {
        let current_dir = env::current_dir().expect("Failed to get return directory");
        env::set_current_dir(self.path.clone()).unwrap();

        let child = process::Command::new("git")
            .args(["remote", "--verbose"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to create process");

        let output = child.wait_with_output().unwrap();
        let output = String::from_utf8_lossy(&output.stdout);

        let lines_lines = output.lines();
        let mut lines = vec![];
        for line in lines_lines {
            lines.push(
                line.to_string()
                    .split_ascii_whitespace()
                    .next()
                    .unwrap()
                    .to_string(),
            );
        }

        // Return to current dir
        env::set_current_dir(current_dir).unwrap();

        lines.dedup();

        lines
    }

    pub fn add_remote(&mut self, name: &str, url: &str) {
        let current_dir = env::current_dir().expect("Failed to get return directory");
        env::set_current_dir(self.path.clone()).unwrap();

        let status = Command::new("git")
            .args(["remote", "add", name, url])
            .stdout(Stdio::null())
            .status()
            .unwrap();

        if !status.success() {
            panic!("Failed to add remote from git repository");
        }

        env::set_current_dir(current_dir).unwrap();
    }

    pub fn remove_remote(&mut self, name: &str) {
        let current_dir = env::current_dir().expect("Failed to get return directory");
        env::set_current_dir(self.path.clone()).unwrap();

        let status = Command::new("git")
            .args(["remote", "remove", name])
            .status()
            .unwrap();

        if !status.success() {
            panic!("Failed to remove remote from git repository");
        }

        env::set_current_dir(current_dir).unwrap();
    }

    pub fn pull(&mut self, remote: &str) {
        let current_dir = env::current_dir().expect("Failed to get return directory");
        env::set_current_dir(self.path.clone()).unwrap();

        let status = Command::new("git")
            .args(["pull", remote, "main", "--quiet"])
            .status()
            .unwrap();

        if !status.success() {
            panic!("Failed to pull changes from remote: {}", remote);
        }

        env::set_current_dir(current_dir).unwrap();
    }
}
