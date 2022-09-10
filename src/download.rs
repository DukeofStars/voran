use std::{
    cmp::min,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

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

// struct DownloadProgress<'a, R: std::io::Read> {
//     inner: R,
//     progress_bar: &'a ProgressBar,
// }

// impl<'a, R: std::io::Read> Read for DownloadProgress<'a, R> {
//     fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
//         self.inner.read(buf).map(|n| {
//             self.progress_bar.inc(n as u64);
//             n
//         })
//     }
// }

// pub async fn download(
//     url: &str,
//     folder: PathBuf,
//     pb: &ProgressBar,
// ) -> Result<PathBuf, ExitFailure> {
//     let url = Url::parse(url)?;
//     let client = Client::new();

//     let total_size = {
//         let resp = client.head(url.as_str()).send().await?;
//         if resp.status().is_success() {
//             resp.headers()
//                 .get(header::CONTENT_LENGTH)
//                 .and_then(|ct_len| ct_len.to_str().ok())
//                 .and_then(|ct_len| ct_len.parse().ok())
//                 .unwrap_or(0)
//         } else {
//             return Err(failure::err_msg(format!(
//                 "Couldn't download URL: {}. Error: {:?}",
//                 url,
//                 resp.status(),
//             ))
//             .into());
//         }
//     };

//     let mut request = client.get(url.as_str());
//     pb.set_length(total_size);
//     pb.set_style(
//         ProgressStyle::default_bar()
//             .template(
//                 "{spinner} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
//             )?
//             .progress_chars("#>-"),
//     );

//     let file = folder.join(Path::new(
//         url.path_segments()
//             .and_then(|segments| segments.last())
//             .unwrap_or("tmp.bin"),
//     ));

//     if file.exists() {
//         let size = file.metadata()?.len() - 1;
//         request = request.header(header::RANGE, format!("bytes={}-", size));
//         pb.inc(size);
//     }

//     // let bytes = request.send().await?.bytes().await?;
//     // let mut source = DownloadProgress {
//     //     progress_bar: pb,
//     //     inner: bytes.as_ref(),
//     // };

//     let mut res = reqwest::get(url).await?;

//     let mut index: u64 = 0;

//     while let Some(chunk) = res.chunk().await? {
//         index += chunk.len() as u64;
//         pb.set_position(index);
//     }

//     // let mut source = DownloadProgress {
//     //     progress_bar: pb,
//     //     inner: res.chunk().await?.unwrap(),
//     // }

//     pb.finish();

//     let mut dest = fs::OpenOptions::new()
//         .create(true)
//         .append(true)
//         .open(&file)?;

//     // let _ = copy(&mut source, &mut dest)?;

//     Ok(file)
// }
