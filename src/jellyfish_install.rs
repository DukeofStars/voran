use std::{
    fs::{self, File},
    os::windows,
    path::{Path, PathBuf},
};

use flate2::read::GzDecoder;
use tar::Archive;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
pub trait JellyFishInstaller {
    fn extract(&self, out: PathBuf) -> Result<(), failure::Error>;
    fn link_to(&self, out: PathBuf, bin_path: PathBuf) -> Result<(), failure::Error>;
}

/// Used to install JellyFish packages.
pub struct BasicJellyFishInstaller {
    input_file: PathBuf,
}

impl BasicJellyFishInstaller {
    /// Create a new JellyFishInstaller.
    pub fn new<P: AsRef<Path> + 'static>(input: P) -> Self {
        Self {
            input_file: input.as_ref().to_path_buf(),
        }
    }
}

impl JellyFishInstaller for BasicJellyFishInstaller {
    fn extract(&self, out: PathBuf) -> Result<(), failure::Error> {
        let file = File::open(&self.input_file)?;
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        archive.unpack(&out)?;
        Ok(())
    }

    fn link_to(&self, out: PathBuf, bin_path: PathBuf) -> Result<(), failure::Error> {
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
        Ok(())
    }
}

/// Extract files from package then link files in /bin to specified bin_path, if link is set to true
pub fn install_to<P: AsRef<Path> + 'static>(
    installer: &impl JellyFishInstaller,
    out: P,
    bin_path: P,
    link: bool,
) -> Result<(), failure::Error> {
    if !out.as_ref().exists() {
        fs::create_dir_all(&out)?;
    }
    if !out.as_ref().is_dir() {
        return Err(failure::err_msg("Output directory must be a folder"));
    }

    // Extract the compressed file.
    installer.extract(out.as_ref().to_path_buf())?;

    // Make a symbolic link with the bin files inside bin_path
    if link {
        installer.link_to(out.as_ref().to_path_buf(), bin_path.as_ref().to_path_buf())?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn install_to_calls_right_functions() {
        let mut mock = MockJellyFishInstaller::default();
        mock.expect_extract().times(1).returning(|_| Ok(()));
        mock.expect_link_to().times(1).returning(|_, _| Ok(()));
        install_to(&mock, "out/", "bin/", true).unwrap();
        mock.checkpoint();

        // Cleanup
        fs::remove_dir_all("out/").unwrap();
    }
}
