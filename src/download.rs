use std::{
    cmp::min,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

/// Download file using a reqwest::Client, from url and put the contents at path.
///
/// ```rust
/// let file = "google_index.html";
/// let url = "https://google.com/index.html";
/// download_file(&Client::new(), url, path).expect("Failed to download file");
/// ```
pub async fn download_file<P: AsRef<Path>>(
    client: &Client,
    url: &str,
    path: P,
) -> Result<PathBuf, failure::Error> {
    // Reqwest setup
    let res = client
        .get(url)
        .send()
        .await
        .or(Err(failure::err_msg(format!(
            "Failed to GET from '{}'",
            url
        ))))?;
    let total_size = res.content_length().ok_or(failure::err_msg(format!(
        "Failed to get content length from '{}'",
        &url
    )))?;

    // Indicatif setup
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
        .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", url));

    // download chunks
    let mut file = File::create(&path).or(Err(failure::err_msg(format!(
        "Failed to create file '{}'",
        path.as_ref().to_str().unwrap()
    ))))?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(failure::err_msg(format!(
            "Error while downloading file"
        ))))?;
        file.write_all(&chunk).or(Err(failure::err_msg(format!(
            "Error while writing to file"
        ))))?;
        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        pb.set_position(new);
    }

    pb.finish_with_message(format!(
        "Downloaded {} to {}",
        url,
        path.as_ref().to_str().unwrap()
    ));

    Ok(path.as_ref().to_path_buf())
}
