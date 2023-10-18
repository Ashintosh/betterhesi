use std::fs::File;
use std::path::Path;
use tokio::sync::Semaphore;
use std::sync::{Arc, Mutex};
use compress_tools::{self, Ownership};


/// Extract a list of archives to the specified path.
///
/// This function extracts multiple archives in parallel using Tokio tasks. It takes a list of archive file
/// names and a target path for the extractions, while tracking the extraction progress.
///
/// # Arguments
///
/// * `archive_list` - A vector of archive file names to be extracted.
/// * `path` - The path where the archives will be extracted.
///
/// # Returns
///
/// * `Result<(), String>` - `Ok(())` if all extractions are successful, or an error message on failure.
pub async fn archive_list(archive_list: &Vec<String>, path: String) -> Result<(), String> {
    let archive_list = Arc::new(Mutex::new(archive_list));
    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = Vec::new();

    for archive_guard in archive_list.lock().iter() {
        for archive in archive_guard.iter() {
            let archive = archive.to_string(); // Clone the specific String
            let mut path = path.clone();
            if path.ends_with('/') { path.pop(); }
            let semaphore = Arc::clone(&semaphore);

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.expect("Semaphore error");
                if let Err(e) = extract_archive(&archive, &path).await {
                    return Err(format!("Error extracting {}: {}", &archive, e));
                }
                Ok(())
            });
            handles.push(handle);
        }
    }

    // Wait for all extraction tasks to complete.
    for handle in handles {
        match handle.await {
            Ok(_t) => (),
            Err(e) => return Err(format!("Task error: {:?}", e))
        }
    }
    Ok(())
}


/// Extract an archive to the specified destination.
///
/// This function extracts an archive located at the specified `archive_path` to the given `dest_path`.
///
/// # Arguments
///
/// * `archive_path` - The path to the archive file to be extracted.
/// * `dest_path` - The destination path where the archive will be extracted.
///
/// # Returns
///
/// * `Result<bool, String>` - `Ok(true)` if extraction is successful, or an error message on failure.
pub async fn extract_archive(archive_path: &String, dest_path: &String) -> Result<bool, String> {
    // Attempt to open the archive file
    let source: File = File::open(archive_path)
        .map_err(|e| format!("Extraction error: {}", e))?;

    // Uncompress the archive to the destination path
    let dest = Path::new(dest_path);
    match compress_tools::uncompress_archive(&mut &source, &dest, Ownership::Preserve) {
        Ok(()) => Ok(true),
        Err(e) => Err(format!("Extraction error: {}", e)),
    }
}