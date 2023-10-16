use tokio::fs;

mod download;
mod extract;
mod directories;


#[tokio::main]
async fn main() {
    // Program configuration
    //let path: &str = "/home/ash/Downloads/hesitest/";
    let cdn: &str = "http://35.190.33.250";

    let Soul: Vec<&str> = vec!["Sol.zip"];
    let csp: Vec<&str> = vec!["CSP.zip"];
    let content_manager: Vec<&str> = vec!["latest.zip"];
    let traffic: Vec<&str> = vec!["traffic.zip"];
    let maps: Vec<&str> = vec!["SRP.zip"];
    let cars: Vec<&str> = vec![
        "cars.zip",
        "cars2.zip"
    ];
    //
    
    // Install Soul
    let archive_path: String = String::new();
    match directories::find_game_directory("steamapps/common/assettocorsa/") {
        Ok(t) => archive_path = t,
        Err(e) => return Err(format!("Could not find directory: {}", e))
    };
    
    println!("Desktop Dir: {}", archive_path);
    match install_packages(cdn, Soul, archive_path).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Error: {}", e))
    };
    //
    
    // Install CSP
    let archive_path: String = format!("{}/",
        directories::get_desktop_directory().unwrap_or_else(|| {
            // Provide a default value or handle the case when the desktop directory is not found.
            "FallbackDirectory".to_string()
    }));
    println!("Desktop Dir: {}", archive_path);
    match install_packages(cdn, csp, archive_path).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Error: {}", e))
    }
    //
}

async fn install_packages(source: &str, package_list: Vec<&str>, archive_path: String) -> Result<(), String> {
    let mut package_urls: Vec<String> = Vec::new();
    let mut archive_list: Vec<String> = Vec::new();
    let temp_dir: String = format!("{}/nohesi/",
        directories::get_temp_directory().unwrap_or_else(|| {
            // Provide a default value or handle the case when the temp directory is not found.
            "FallbackDirectory".to_string()
        }));
    println!("Temp Dir: {}", temp_dir);
    
    if std::path::Path::new(&temp_dir).exists() {
        match fs::remove_dir(&temp_dir).await {
            Ok(_) => (),
            Err(e) => return Err(format!("Could not delete old temp directory: {}", e))
        }
    }
    
    match fs::create_dir(&temp_dir).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Could not create temp directory: {}", e))
    }
    
    for package in &package_list {
        let package_url: String = format!("{}/{}", source, package);
        let archive_dir: String = format!("{}/{}", &temp_dir, package);
        package_urls.push(package_url);
        archive_list.push(archive_dir);
    }
    
    match download::package_list(package_urls, package_list, &temp_dir).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed: {}", e))
    }
    
    match extract::archive_list(archive_list, archive_path).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Download failed: {}", e))
    }
    
    Ok(())
}

fn delete_temp_files() {
    
}