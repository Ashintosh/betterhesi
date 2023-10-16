use reqwest::Client;
use byte_unit::Byte;
use std::error::Error;
use std::sync::{Arc, Mutex};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::{io::AsyncWriteExt, sync::Semaphore};


pub async fn package_list(package_urls: Vec<String>, file_names: Vec<&str>, path: &str) -> Result<(), String> {
    let package_urls = Arc::new(Mutex::new(package_urls));
    let file_names = Arc::new(Mutex::new(file_names));
    let semaphore = Arc::new(Semaphore::new(1)); // Limit to 1 task.
    let handles: Vec<_> = package_urls
        .lock()
        .unwrap()
        .iter()
        .zip(file_names.lock().unwrap().iter())
        .map(|(package_url, file_name)| {
            let package_url = package_url.to_string();
            let file_name = file_name.to_string();
            let full_path = format!("{}{}", path, file_name);
            let semaphore = Arc::clone(&semaphore);

            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.expect("Semaphore error");
                match download_url(&package_url, &full_path, &file_name).await {
                    Ok(_t) => (),
                    Err(e) => return Err(format!("Error downloading {}: {}", file_name, e))
                }
                Ok(())
            })
        })
        .collect();
        
    // Wait for all download tasks to complete.
    for handle in handles {
        match handle.await {
            Ok(_t) => (),
            Err(e) => return Err(format!("Task error: {:?}", e))
        }
    }
    Ok(())
}

async fn download_url(url: &str, path: &str, file_name: &String) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("url: {}", url);
    println!("path: {}", path);
    println!("filename: {}", file_name);
    println!("");
    let client = Client::new();
    let mut response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Failed to download file: {}", response.status()).into());
    }

    let file = tokio::fs::File::create(path)
        .await
        .map_err(|e| format!("Failed to create file: {}", e))?;
    
    let mut writer = tokio::io::BufWriter::new(file);
    let content_length = response.content_length().unwrap_or(0);
    let mut downloaded_bytes = 0;

    let pb = ProgressBar::new(content_length);
    pb.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {msg}")
    .unwrap()
    .progress_chars("##-"));

    while let Some(chunk) = response.chunk().await
        .map_err(|e| format!("Failed to read response chunk: {}", e))?
    {
        writer.write_all(&chunk).await?;
        downloaded_bytes += chunk.len() as u64;
        pb.set_position(downloaded_bytes);
        let formatted_download_bytes = Byte::from_bytes(downloaded_bytes as u128)
            .get_appropriate_unit(false)
            .to_string();
        let formatted_content_bytes = Byte::from_bytes(content_length as u128)
            .get_appropriate_unit(false)
            .to_string();
        pb.set_message(format!("{} | {} / {}", file_name, formatted_download_bytes, formatted_content_bytes));
    }

    pb.finish();
    Ok(())
}