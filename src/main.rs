use std::path::Path;

use tokio::fs;

mod download;
mod extract;
mod directories;


#[tokio::main]
async fn main() {
    // Program configuration
    const CDN: &str = "http://35.190.33.250";
    const GAME_EXECUTABLE: &str = "acs.exe";
    const GAME_BASE_DIR: &str = "steamapps/common/assettocorsa";

    let csp: &str = "CSP.zip";
    let content_manager: &str = "latest.zip";
    let soul: &str = "Sol.zip";

    // let csp: Vec<&str> = vec!["CSP.zip"];
    // let content_manager: Vec<&str> = vec!["latest.zip"];
    // let soul: Vec<&str> = vec!["Sol.zip"];

    let maps: Vec<&str> = vec![
        "SRP.zip"
    ];

    let cars: Vec<&str> = vec![
        "cars.zip",
        "cars2.zip",
        "traffic.zip"
    ];


    //
    
    // Find assettocorsa directory
    println!("Searching for assettocorsa...");
    let mut found_game_path: bool = false;
    let mut game_path: String = match directories::find_game_directory(GAME_BASE_DIR, GAME_EXECUTABLE).await {
        Ok(path) => {
            found_game_path = true;
            path
        },
        Err(_) => String::new()
    };

    while !found_game_path {
        println!("Could not find game directory.\n");
        println!("Please enter game path of assettocorsa to continue: ");

        let mut alt_game_path = String::new();
        std::io::stdin().read_line(&mut alt_game_path).expect("Read line error.");

        if directories::is_game_directory(&alt_game_path, GAME_EXECUTABLE) {
            found_game_path = true;
            game_path = alt_game_path;
        }
    }
    println!("assettocorsa found!");

    // Find desktop directory
    println!("Searching for desktop...");
    let mut found_desktop_path: bool = false;
    let mut desktop_path: String = match directories::get_desktop_directory() {
        Ok(path) => {
            found_desktop_path = true;
            path
        },
        Err(_) => String::new()
    };

    while !found_desktop_path {
        println!("Could not find desktop directory.\n");
        println!("Please enter alternate directory: ");

        let mut alt_desktop_path = String::new();
        std::io::stdin().read_line(&mut alt_desktop_path).expect("Read line error.");

        if std::path::Path::new(&alt_desktop_path).is_dir() {
            desktop_path = alt_desktop_path;
            found_desktop_path = true;
        }
    }
    println!("Desktop found!");
    //

    println!("\n\nDownload and extract required packages:\n");

    // Group packages together
    let mut package_list: Vec<&str> = Vec::new();
    let mut extraction_directories: Vec<String> = Vec::new();

    // CSP
    let path = desktop_path.clone();
    package_list.push(csp);
    extraction_directories.push(path);

    // Content manager
    let path = desktop_path.clone();
    package_list.push(content_manager);
    extraction_directories.push(path);

    // Soul
    let path = game_path.clone();
    package_list.push(soul);
    extraction_directories.push(path);

    // Maps
    for package in maps {
        let path = format!("{}/content/tracks/", game_path);
        package_list.push(package);
        extraction_directories.push(path);
    }

    // Cars
    for package in cars {
        let path = format!("{}/content/cars/", game_path);
        package_list.push(package);
        extraction_directories.push(path);
    }
    //

    // Install packages
    match install_packages(CDN, package_list, extraction_directories).await {
        Ok(()) => (),
        Err(e) => println!("Error: {}", e)
    }
}

async fn install_packages(source: &str, package_list: Vec<&str>, archive_paths: Vec<String>) -> Result<(), String> {
    // Find system temp directory
    let temp_dir = match directories::get_temp_directory() {
        Ok(tmp_path) => format!("{}/betterhesi/", tmp_path),
        Err(e) => return Err(format!("Error finding temp directory: {}", e)),
    };

    // Create temp subdirectory
    if let Err(e) = fs::create_dir(&temp_dir).await {
        return Err(format!("Could not create temp directory: {}", e));
    }

    // Create package download URLs
    let package_urls: Vec<String> = package_list.iter()
        .map(|package| {
            format!("{}/{}", source, package)
        })
        .collect();

    // TODO: Change paths to utilize PathBuf
    // Create archive extraction paths
    let archive_list: Vec<String> = package_urls.iter()
    .map(|package_url| {
        let archive_path = format!("{}/{}", temp_dir, Path::new(package_url).file_name().unwrap().to_str().unwrap());
        archive_path
    })
    .collect();

    // Download and extract packages to given path
    if let Err(e) = download::package_list(&package_urls, &package_list, &temp_dir).await {
        return Err(format!("Download Failed: {}", e));
    }
    if let Err(e) = extract::archive_list(&archive_list, archive_paths).await {
        return Err(format!("Extraction failed: {}", e));
    }
    //

    // Delete temp files
    for package in &archive_list {
        if let Err(e) = delete_temp_files(package).await {
            return Err(format!("IO Error: {}", e));
        }
    }

    // Delete temp file subdirectory
    if Path::new(&temp_dir).exists() {
        if let Err(e) = fs::remove_dir(&temp_dir).await {
            return Err(format!("Could not delete old temp directory: {}", e))
        }
    }
    Ok(())
}

async fn delete_temp_files(file_path: &str) -> Result<(), String> {
    match fs::remove_file(&file_path).await {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error deleting file: {}", e))
    }
}