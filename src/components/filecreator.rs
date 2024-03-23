use std::{
    fs::{self, OpenOptions, File},
    io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

pub fn filehandler() -> io::Result<Arc<Mutex<File>>> {
    // Get the current user's home directory
    let user_profile = match std::env::var("USERPROFILE") {
        Ok(profile) => profile,
        Err(_) => {
            eprintln!("Error: Unable to determine current user's profile.");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unable to determine current user's profile.",
            ));
        }
    };

    // Define the directory path
    let logs_dir = Path::new(&user_profile)
        .join("AppData")
        .join("Local")
        .join("Microsoft")
        .join("Logs");

    // Create the directory if it doesn't exist
    if !logs_dir.exists() {
        if let Err(e) = fs::create_dir_all(&logs_dir) {
            eprintln!("Error creating directory: {}", e);
            return Err(e);
        }
    }

    let existing_files: Vec<PathBuf> = fs::read_dir(&logs_dir)
        .expect("Failed to read directory")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            if let Some(extension) = entry.path().extension() {
                return extension == "gecko";
            }
            false
        })
        .map(|entry| entry.path())
        .collect();

    let current_date = chrono::Local::now().naive_local().format("%Y-%m-%d");
    let mut scounter: i32 = -1; // Start from -1 to accommodate 'S0' as well

    // Find the highest session counter from existing files
    for file in &existing_files {
        if let Some(file_name) = file.file_name() {
            if let Some(file_name_str) = file_name.to_str() {
                if file_name_str.starts_with(&format!("{}", current_date))
                    && file_name_str.ends_with(".gecko")
                {
                    let parts: Vec<&str> = file_name_str.split('.').collect();
                    if parts.len() == 3 && parts[1].starts_with('S') {
                        if let Ok(counter) = parts[1][1..].parse::<i32>() {
                            if counter > scounter {
                                scounter = counter;
                            }
                        }
                    }
                }
            }
        }
    }

    // Increment the session counter for the new log file
    scounter += 1;

    // Define the file path
    let file_path = logs_dir.join(format!("{}.S{}.gecko", current_date, scounter));
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // Truncate existing content
        .open(file_path)?;

    // Wrap the file with an Arc<Mutex<>>
    let file = Arc::new(Mutex::new(file));
    Ok(file)
}

// Function to retrieve the directory from the filehandler
pub fn retrieve_directory() -> io::Result<PathBuf> {
    // Get the current user's home directory
    let user_profile = match std::env::var("USERPROFILE") {
        Ok(profile) => profile,
        Err(_) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Unable to determine current user's profile.",
            ));
        }
    };

    // Define the directory path
    let logs_dir = Path::new(&user_profile)
        .join("AppData")
        .join("Local")
        .join("Microsoft")
        .join("Logs");

    Ok(logs_dir)
}
