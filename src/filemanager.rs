use directories::ProjectDirs;
use std::{fs::create_dir_all, path::PathBuf};

/// Creates and then returns the path to a suitable location for application data to be stored.
/// If the path already exists, just return the path.
pub fn get_application_directory() -> Result<PathBuf, ()> {
    let project_dir = match ProjectDirs::from("com", "hchap1", "resonate") {
        Some(project_dir) => project_dir,
        None => return Err(())
    };

    let path = project_dir.data_dir().to_path_buf();
    let _ = create_dir_all(&path);
    Ok(path)
}
