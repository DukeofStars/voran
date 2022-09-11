use std::{
    fs::{self, File},
    os::windows,
    path::PathBuf,
};

use flate2::read::GzDecoder;
use tar::Archive;

/// Used to install JellyFish packages.
pub struct JellyFishInstaller {
    input_file: PathBuf,
}

impl JellyFishInstaller {
    /// Create a new JellyFishInstaller.
    pub fn new(input: PathBuf) -> Self {
        Self { input_file: input }
    }

    /// Extract files from package then link files in /bin to specified bin_path, if link is set to true.
    pub fn install_to(
        &self,
        out: PathBuf,
        bin_path: PathBuf,
        link: bool,
    ) -> Result<(), failure::Error> {
        if !out.exists() {
            fs::create_dir_all(&out)?;
        }
        if !out.is_dir() {
            return Err(failure::err_msg("Output directory must be a folder"));
        }

        // Extract the compressed file.
        let file = File::open(&self.input_file)?;
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        archive.unpack(&out)?;

        // Make a symbolic link with the bin files inside bin_path
        if link {
            let bin = out.join("bin");
            if !bin.exists() {
                // Package doesn't have any binaries
                return Ok(());
            }

            if !bin_path.exists() {
                fs::create_dir_all(&bin_path)?;
            }
            if !bin_path.is_dir() {
                return Err(failure::err_msg("Bin folder must be a directory"));
            }

            // Iterate over bin files
            for bin_file in bin
                .read_dir()
                .expect("Failed to read package bin directory")
            {
                let file = bin_file?;

                let link = bin_path.join(file.file_name());
                if link.exists() {
                    fs::remove_file(link).unwrap();
                }
                #[cfg(windows)]
                {
                    if file.path().is_dir() {
                        windows::fs::symlink_dir(file.path(), bin_path.join(file.file_name()))?;
                    } else if file.path().is_file() {
                        windows::fs::symlink_file(file.path(), bin_path.join(file.file_name()))?;
                    }
                }
            }
        }

        Ok(())
    }
}
