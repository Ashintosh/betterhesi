use std::fs;
use std::env;
use walkdir::WalkDir;


pub fn find_game_directory(target: &str) -> Result<String, bool> {
    let drives = get_available_drives();
    for drive in drives {
        // Iterate through all directories on the drive
        for entry in WalkDir::new(&drive).into_iter().filter_map(|e| e.ok()) {
            let file_name = entry.file_name().to_string_lossy();
            
            if entry.file_type().is_dir() && file_name == target {
                println!("Found game directory at: {:?}", entry.path());
                return Ok(entry.path().to_string_lossy().to_string());
            }
        }
    }

    println!("Failed to find game directory! Make sure it is installed.");
    Err(false)
}

pub fn get_desktop_directory() -> Option<String> {
    if cfg!(windows) {
        env::var_os("USERPROFILE").map(|userprofile| {
            let mut path = std::path::PathBuf::from(userprofile);
            path.push("Desktop");
            path.to_string_lossy().into_owned()
        })
    } else {
        env::var_os("HOME").map(|home| {
            let mut path = std::path::PathBuf::from(home);
            path.push("Desktop");
            path.to_string_lossy().into_owned()
        })
    }
}

pub fn get_temp_directory() -> Option<String> {
    match env::temp_dir().to_str() {
        Some(temp_dir_str) => Some(temp_dir_str.to_string()),
        None => None,
    }
}

fn get_available_drives() -> Vec<String> {
    if cfg!(windows) {
        // On Windows, iterate through drive letters
        (b'A'..=b'Z')
            .map(|drive_letter| format!("{}:\\", drive_letter as char))
            .filter(|drive| fs::metadata(&drive).is_ok())
            .collect()
    } else {
        // On Unix-based systems, iterate through mount points
        let root_dir = "/";
        fs::read_dir(root_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path().to_string_lossy().to_string())
            .collect()
    }
}