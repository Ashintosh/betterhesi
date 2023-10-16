use std::fs::File;
use std::path::Path;
use tokio::sync::Semaphore;
use std::sync::{Arc, Mutex};
use compress_tools::{self, Ownership};

pub async fn archive_list(archive_list: Vec<String>, path: String) -> Result<(), String> {
    let archive_list = Arc::new(Mutex::new(archive_list));
    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = Vec::new();

    for archive_guard in archive_list.lock().iter() {
        for archive in archive_guard.iter() {
            let archive = archive.to_string(); // Clone the specific String
            let path = path.clone();
            let semaphore = Arc::clone(&semaphore);

            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.expect("Semaphore error");
                match extract_archive(&archive, &path).await {
                    Ok(_t) => (),
                    Err(e) => return Err(format!("Error extracting {}: {}", &archive, e))
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

async fn extract_archive(archive_path: &String, dest_path: &String) -> Result<bool, String> {
    match File::open(archive_path) {
        Ok(_t) => (),
        Err(e) => return Err(format!("Extraction error: {}", e))
    };
    
    let source: File = File::open(archive_path).unwrap();
    let dest = Path::new(&dest_path);

    match compress_tools::uncompress_archive(&mut &source, &dest, Ownership::Preserve) {
        Ok(()) => return Ok(true),
        Err(e) => return Err(format!("Extraction error: {}", e)),
    }
}