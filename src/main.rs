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

    let csp: Vec<&str> = vec!["CSP.zip"];
    let content_manager: Vec<&str> = vec!["latest.zip"];
    let soul: Vec<&str> = vec!["Sol.zip"];
    let maps: Vec<&str> = vec!["SRP.zip"];
    let cars: Vec<&str> = vec![
        "cars.zip",
        "cars2.zip",
        "traffic.zip"
    ];
    //
    
    // Find assettocorsa directory
    println!("Searching for assettocorsa...");
    let mut game_path: String = String::new();
    match directories::find_game_directory(GAME_BASE_DIR, GAME_EXECUTABLE).await {
        Ok(path) => {
            game_path = path;
        },
        Err(e) => {
            let mut found_game_path: bool = false;
            while !found_game_path {
                println!("Could not find game directory: Result: {}\n", e);
                println!("Please enter game path of assettocorsa to continue: ");

                let mut alt_game_path = String::new();
                std::io::stdin().read_line(&mut alt_game_path).expect("Read line error.");

                if directories::is_game_directory(&alt_game_path, GAME_EXECUTABLE) {
                    game_path = alt_game_path;
                    found_game_path = true;
                }
            }
        }
    }
    println!("assettocorsa found!");

    // Find desktop directory
    println!("Searching for desktop...");
    let mut desktop_path: String = String::new();
    match directories::get_desktop_directory() {
        Ok(path) => desktop_path = path,
        Err(e) => {
            let mut found_desktop_path: bool = false;
            while !found_desktop_path {
                println!("Could not find desktop directory: Result: {}\n", e);
                println!("Please enter alternate directory: ");

                let mut alt_desktop_path = String::new();
                std::io::stdin().read_line(&mut alt_desktop_path).expect("Read line error.");

                if std::path::Path::new(&alt_desktop_path).is_dir() {
                    desktop_path = alt_desktop_path;
                    found_desktop_path = true;
                }
            }
        }
    }
    println!("Desktop found!");
    //

    println!("\n\nDownload and extract required packages:\n");

    /*
    // Group packages together
    let tmp_packages: Vec<Vec<&str>> = vec![
        csp.clone(),
        content_manager.clone(),
        soul.clone(),
        maps.clone(),
        cars.clone()
    ];
    
    let test: Vec<Vec<&str>> = vec![

    ];

    let package_count: i32 = tmp_packages.iter().map(|v| v.len() as i32).sum();

    let mut package_list: Vec<&str> = Vec::new();
    package_list.extend(&csp);
    package_list.extend(&content_manager);
    package_list.extend(&soul);
    package_list.extend(&maps);
    package_list.extend(&cars);

    let package_list_extraction_dirs: Vec<&str> = Vec::new();
    */
    // Install CSP
    let path = desktop_path.clone();
    if let Err(e) = install_packages(CDN, csp, path).await {
        println!("Error: {}", e);
    }
    //

    // Install Content Manager
    let path = desktop_path.clone();
    if let Err(e) = install_packages(CDN, content_manager, path).await {
        println!("Error: {}", e);
    }
    //

    // Install Soul
    let path = game_path.clone();
    if let Err(e) = install_packages(CDN, soul, path).await {
        println!("Error: {}", e);
    }
    //

    // Install maps
    let path = format!("{}/content/tracks/", game_path);
    if let Err(e) = install_packages(CDN, maps, path).await {
        println!("Error: {}", e);
    }
    //

    // Install cars
    let path = format!("{}/content/cars/", game_path);
    if let Err(e) = install_packages(CDN, cars, path).await {
        println!("Error: {}", e);
    }
    //
}

async fn install_packages(source: &str, package_list: Vec<&str>, archive_path: String) -> Result<(), String> {
    let mut package_urls: Vec<String> = Vec::new();
    let mut archive_list: Vec<String> = Vec::new();
    let temp_dir: String = format!("{}/betterhesi/",
        directories::get_temp_directory().unwrap_or_else(|| {
            // Provide a default value or handle the case when the temp directory is not found.
            "FallbackDirectory".to_string()
        }));
    
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
        let archive_dir: String = format!("{}{}", &temp_dir, package);
        package_urls.push(package_url);
        archive_list.push(archive_dir);
    }
    
    match download::package_list(&package_urls, &package_list, &temp_dir).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed: {}", e))
    }
    
    match extract::archive_list(&archive_list, archive_path).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Extraction failed: {}", e))
    }

    for package in &archive_list {
        match delete_temp_files(package.as_str()).await {
            Ok(_) => (),
            Err(e) => return Err(format!("Error: {}", e))
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