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
/// ```rust_async
/// let path = "google_index.html";
/// let url = "https://google.com/index.html";
/// voran::download_file(&reqwest::Client::new(), url, path).expect("Failed to download file");
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
        .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap()
        .progress_chars("#>-"));

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

    pb.finish();

    Ok(path.as_ref().to_path_buf())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use reqwest::Client;
    use tokio::fs::{self, OpenOptions};

    use crate::download_file;

    #[tokio::test]
    async fn download_full_file() {
        let url = "https://www.dundeecity.gov.uk/sites/default/files/publications/civic_renewal_forms.zip";
        let path = "testing/test.txt";
        if !PathBuf::from(&path).parent().unwrap().exists() {
            fs::create_dir_all(PathBuf::from(&path).parent().unwrap())
                .await
                .unwrap();
        }
        download_file(&Client::new(), url, &path).await.unwrap();
        let mut options = OpenOptions::default();
        options.read(true);
        let file = options.open(&path).await.unwrap();
        let len = file.metadata().await.unwrap().len();
        assert_eq!(1092867, len);
        fs::remove_dir_all(PathBuf::from(path).parent().unwrap())
            .await
            .unwrap();
    }
}
