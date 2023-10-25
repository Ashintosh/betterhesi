use reqwest::Client;
use byte_unit::Byte;
use std::error::Error;
use tokio::io::AsyncWriteExt;
use indicatif::{ProgressBar, ProgressStyle};


/// Download a list of packages from specified URLs to the given path.
///
/// This function downloads packages in parallel using Tokio tasks. It takes a list
/// of package URLs, associated file names, and a target path for the downloads.
///
/// # Arguments
///
/// * `package_urls` - A vector of package URLs to download.
/// * `file_names` - A vector of corresponding file names.
/// * `path` - The path where packages will be downloaded.
///
/// # Returns
///
/// * `Result<(), String>` - `Ok(())` if all downloads are successful, or an error message on failure.
pub async fn package_list(package_urls: &[String], file_names: &[&str], path: &str) -> Result<(), String> {
    for (package_url, file_name) in package_urls.iter().zip(file_names.iter()) {
        let package_url = package_url.to_string();
        let file_name = file_name.to_string();
        let full_path = format!("{}/{}", path, file_name);

        if let Err(e) = download_url(&package_url, &full_path, &file_name).await {
            return Err(format!("Error downloading {}: {}", file_name, e));
        }
    }

    Ok(())
}


/// Download a file from a given URL and track the download progress.
///
/// This function downloads a file from the specified URL and saves it to the provided path,
/// while tracking the download progress using a progress bar.
///
/// # Arguments
///
/// * `url` - The URL of the file to be downloaded.
/// * `path` - The path where the downloaded file will be saved.
/// * `file_name` - The name of the downloaded file.
///
/// # Returns
///
/// * `Result<(), Box<dyn Error + Send + Sync>>` - `Ok(())` if the download is successful, or an error on failure.
async fn download_url(url: &str, path: &str, file_name: &String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let client = Client::new();
    let mut response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to download file: {}", response.status()).into());
    }

    // Create path to write bytes
    let file = tokio::fs::File::create(path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    let mut writer = tokio::io::BufWriter::new(file);
    let content_length = response.content_length().unwrap_or(0);

    // Initialize progress bar
    let pb = ProgressBar::new(content_length);
    pb.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
    .unwrap()
    .progress_chars("##-"));

    // Download the content in chunks
    let mut downloaded_bytes = 0;
    while let Some(chunk) = response.chunk().await
        .map_err(|e| format!("Failed to read response chunk: {}", e))?
    {
        writer.write_all(&chunk).await?;

        // Update progress bar
        downloaded_bytes += chunk.len() as u64;
        pb.set_position(downloaded_bytes);

        let formatted_download_bytes = Byte::from_bytes(downloaded_bytes as u128)
            .get_appropriate_unit(false)
            .to_string();
        let formatted_content_bytes = Byte::from_bytes(content_length as u128)
            .get_appropriate_unit(false)
            .to_string();
        pb.set_message(format!("({}) {}/{}",
            file_name,
            formatted_download_bytes,
            formatted_content_bytes
        ));
    }

    pb.finish();
    Ok(())
}