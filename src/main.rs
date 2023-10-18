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
    package_list.push(csp[0]);
    extraction_directories.push(path);

    // Content manager
    let path = desktop_path.clone();
    package_list.push(content_manager[0]);
    extraction_directories.push(path);

    // Soul
    let path = game_path.clone();
    package_list.push(soul[0]);
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

    for package in &package_list {
        println!("{package}");
    }

    match install_packages(CDN, package_list, extraction_directories).await {
        Ok(()) => (),
        Err(e) => println!("Error: {}", e)
    }
}

async fn install_packages(source: &str, package_list: Vec<&str>, archive_paths: Vec<String>) -> Result<(), String> {
    let mut package_urls: Vec<String> = Vec::new();
    let mut archive_list: Vec<String> = Vec::new();
    let temp_dir: String = match directories::get_temp_directory() {
        Ok(tmp_path) => format!("{}/betterhesi/", tmp_path),
        Err(e) => panic!("Error finding temp directory: {}", e)
    };
    
    match fs::create_dir(&temp_dir).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Could not create temp directory: {}", e))
    }
    
    for package in &package_list {
        println!("install_packages: {package}");
        let package_url: String = format!("{}/{}", source, package);
        let archive_dir: String = format!("{}{}", &temp_dir, package);
        package_urls.push(package_url);
        archive_list.push(archive_dir);
    }
    
    match download::package_list(&package_urls, &package_list, &temp_dir).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Failed: {}", e))
    }
    
    match extract::archive_list(&archive_list, archive_paths).await {
        Ok(_) => (),
        Err(e) => return Err(format!("Extraction failed: {}", e))
    }
    
    for package in &archive_list {
        match delete_temp_files(package.as_str()).await {
            Ok(_) => (),
            Err(e) => return Err(format!("Error: {}", e))
        }
    }

    if std::path::Path::new(&temp_dir).exists() {
        match fs::remove_dir(&temp_dir).await {
            Ok(_) => (),
            Err(e) => return Err(format!("Could not delete old temp directory: {}", e))
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