use std::fs;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use walkdir::WalkDir;


/// Find the game directory by searching through available drives in parallel.
///
/// This function asynchronously searches through available drives to find a game
/// directory that contains the specified executable. It returns the path of the
/// game directory if found, or an error if no valid directory is found.
///
/// # Arguments
///
/// * `target` - The name of the target directory within the game directory.
/// * `executable` - The name of the game's executable file.
///
/// # Returns
///
/// * `Result<String, bool>` - The path of the game directory if found, or `Err(false)` if not found.
#[allow(clippy::implicit_hasher)]
pub async fn find_game_directory(target: &str, executable: &str) -> Result<String, bool> {
    // Get a list of available drives to search
    let drives = get_available_drives();
    let mut handles = vec![];

    // Spawn asynchronous tasks to search through drives in parallel
    for drive in drives {
        let target = target.to_string();
        let executable = executable.to_string();

        let handle = tokio::spawn(async move {
            let drive_path = Path::new(&drive);
            // Check if the drive exists and is a directory
            if drive_path.exists() && drive_path.is_dir() {
                for entry in WalkDir::new(drive).into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let path_str = format!("{}/{}", path.to_string_lossy(), &target);
                        // Check if the directory contains the game executable
                        if is_game_directory(&path_str, &executable) {
                            return Ok(path_str);
                        }
                    }
                }
            }
            Err(false)
        });
        handles.push(handle);
    }

    // Await results of asynchronous tasks and return the first valid game directory found
    for handle in handles {
        let result = handle.await.unwrap();
        if let Ok(game_directory) = result {
            return Ok(game_directory);
        }
    }

    // Return an error if no valid game directory is found
    Err(false)
}


/// Check if a directory is a game directory by verifying the existence of a specific executable file.
///
/// This function determines if a directory is a game directory by checking the existence of a
/// specified executable file within the directory.
///
/// # Arguments
///
/// * `path` - The path to the directory to check.
/// * `executable` - The name of the executable file (e.g., "game.exe") to check for in the directory.
///
/// # Returns
///
/// * `bool` - `true` if the directory is a game directory, `false` otherwise.
pub fn is_game_directory(path: &str, executable: &str) -> bool {
    // Create the full path to the executable within the directory
    let executable_path = format!("{}/{}", path, executable);

    // Check if the directory and the executable file exist
    let directory_exists = fs::metadata(&path).is_ok();
    let executable_exists = fs::metadata(&executable_path).is_ok();

    // Return `true` if both the directory and executable exist, indicating it's a game directory
    directory_exists && executable_exists
}


/// Get the user's desktop directory path.
///
/// This function retrieves the path to the user's desktop directory. It is OS-dependent,
/// as it checks for the environment variable USERPROFILE on Windows and HOME on other platforms.
///
/// # Returns
///
/// * `Result<String, String>` - The path to the desktop directory if found, or an error message if not found.
pub fn get_desktop_directory() -> Result<String, String> {
    let env_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let sub_directory = "Desktop";

    match env::var_os(env_var) {
        Some(home) => {
            let path = std::path::PathBuf::from(home).join(sub_directory);
            Ok(path.to_string_lossy().into_owned())
        },
        None => Err(format!("{} environment variable not found", env_var)),
    }
}


/// Get the system's temporary directory path.
///
/// This function retrieves the path to the system's temporary directory.
///
/// # Returns
///
/// * `Option<String>` - The path to the temporary directory if available, or `None` if not found.
pub fn get_temp_directory() -> Result<String, String> {
    // Attempt to obtain the temporary directory path as a string.
    match env::temp_dir().to_str() {
        Some(temp_dir_str) => Ok(temp_dir_str.to_string()),
        None => Err("Failed to obtain temporary directory path".to_string()),
    }
}


/// Get a list of available drives or mount points based on the operating system.
///
/// This function returns a list of available drives (Windows) or mount points (Unix-based) on the system.
///
/// # Returns
///
/// * `Vec<String>` - A list of available drives or mount points.
pub fn get_available_drives() -> Vec<String> {
    if cfg!(windows) {
        // On Windows, iterate through drive letters
        (b'A'..=b'Z')
            .map(|drive_letter| format!("{}:\\", drive_letter as char))
            .filter(|drive| fs::metadata(&drive).is_ok())
            .collect()
    } else {
        // On Unix-based systems, iterate through mount points
        let root_dir = PathBuf::from("/");
        match fs::read_dir(&root_dir) {
            Ok(entries) => {
                let mut available_drives = Vec::new();
                for entry in entries {
                    if let Ok(entry) = entry {
                        available_drives.push(entry.path().to_string_lossy().to_string());
                    }
                }
                available_drives
            }
            Err(_) => Vec::new(),
        }
    }
}