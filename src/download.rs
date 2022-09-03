use std::{
    fs,
    io::{self, copy, Read},
    path::{Path, PathBuf},
};

use exitfailure::ExitFailure;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Url;
use reqwest::{header, Client};

struct DownloadProgress<R: std::io::Read> {
    inner: R,
    progress_bar: ProgressBar,
}

impl<R: std::io::Read> Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

pub async fn download(url: &str, folder: PathBuf) -> Result<PathBuf, ExitFailure> {
    let url = Url::parse(url)?;
    let client = Client::new();

    let total_size = {
        let resp = client.head(url.as_str()).send().await?;
        if resp.status().is_success() {
            resp.headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|ct_len| ct_len.to_str().ok())
                .and_then(|ct_len| ct_len.parse().ok())
                .unwrap_or(0)
        } else {
            return Err(failure::err_msg(format!(
                "Couldn't download URL: {}. Error: {:?}",
                url,
                resp.status(),
            ))
            .into());
        }
    };

    let mut request = client.get(url.as_str());
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
                 .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
                 .progress_chars("#>-"));

    let file = folder.join(Path::new(
        url.path_segments()
            .and_then(|segments| segments.last())
            .unwrap_or("tmp.bin"),
    ));

    if file.exists() {
        let size = file.metadata()?.len() - 1;
        request = request.header(header::RANGE, format!("bytes={}-", size));
        pb.inc(size);
    }

    let bytes = request.send().await?.bytes().await?;
    let mut source = DownloadProgress {
        progress_bar: pb,
        inner: bytes.as_ref(),
    };

    let mut dest = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)?;

    let _ = copy(&mut source, &mut dest)?;

    Ok(file)
}
