use crate::error::{
    doas_cat_permission_denied, doas_unavailable, doas_validation_error, interrupted,
    invalid_editor, Result,
};
use nix::unistd::getuid;
use std::fs;
use std::io::{self, Write};
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the current user ID
pub fn get_current_uid() -> u32 {
    getuid().as_raw()
}

/// Check if a file is owned by the current user
pub fn is_file_owned_by_user(file_path: &Path) -> Result<bool> {
    let metadata = fs::metadata(file_path)?;
    let file_uid = metadata.st_uid();
    let current_uid = get_current_uid();

    Ok(file_uid == current_uid)
}

/// Check if a directory is owned by the current user
pub fn is_dir_owned_by_user(dir_path: &Path) -> Result<bool> {
    let metadata = fs::metadata(dir_path)?;
    let dir_uid = metadata.st_uid();
    let current_uid = get_current_uid();

    Ok(dir_uid == current_uid)
}

/// Check if a directory is writable by current user
pub fn is_dir_writable_by_user(dir_path: &Path) -> Result<bool> {
    let metadata = fs::metadata(dir_path)?;
    let mode = metadata.permissions().mode();
    let dir_uid = metadata.st_uid();
    let current_uid = get_current_uid();

    // Check if directory is writable by user, group, or others
    // But also check if current user is owner
    Ok((mode & 0o222 != 0) && (dir_uid == current_uid))
}

/// Check if a file is writable by current user
pub fn is_file_writable_by_user(file_path: &Path) -> Result<bool> {
    let metadata = fs::metadata(file_path)?;
    let mode = metadata.permissions().mode();
    let file_uid = metadata.st_uid();
    let current_uid = getuid().as_raw();

    // Check if file is writable by user, group, or others
    // But also check if current user is the owner
    Ok((mode & 0o222 != 0) && (file_uid == current_uid))
}

/// Get file metadata using doas if needed
pub fn get_file_metadata_with_doas(file_path: &Path) -> Result<(u32, u32)> {
    let output = Command::new("doas")
        .arg("stat")
        .arg("-c")
        .arg("%u %a")
        .arg(file_path)
        .output()
        .map_err(|_| doas_unavailable())?;

    if !output.status.success() {
        return Err(doas_cat_permission_denied());
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = output_str.trim().split_whitespace().collect();

    if parts.len() >= 2 {
        let uid = parts[0]
            .parse()
            .map_err(|_| doas_validation_error("Invalid UID format"))?;
        let mode = u32::from_str_radix(parts[1], 8).unwrap_or(0o644);

        Ok((uid, mode))
    } else {
        Err(doas_validation_error("Invalid stat output"))
    }
}

/// Check if a path is a directory (ends with /)
pub fn is_directory_path(path: &str) -> bool {
    path.ends_with('/')
}

/// Get the parent directory of a file path
pub fn get_parent_directory(file_path: &Path) -> PathBuf {
    file_path.parent().unwrap_or(Path::new("/")).to_path_buf()
}

/// Read user input from stdin
pub fn read_user_input(prompt: &str) -> Result<String> {
    eprint!("{}", prompt);
    io::stdout().flush().map_err(|_| interrupted())?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|_| interrupted())?;

    Ok(input)
}

/// Check if two files have identical content
pub fn files_match(file1: &Path, file2: &Path) -> Result<bool> {
    let content1 = fs::read(file1)?;
    let content2 = fs::read(file2)?;

    Ok(content1 == content2)
}

/// Check if a file is a doas configuration file
pub fn is_doas_config_file(file_path: &str) -> bool {
    file_path.starts_with("/etc/doas")
        && (file_path == "/etc/doas.conf" || file_path.starts_with("/etc/doas.d/"))
}

/// Get the filename from a path
pub fn get_filename(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| invalid_editor("Invalid file path"))
}

/// Create a safe filename for the copy
pub fn create_copy_filename(original_name: &str) -> String {
    format!("copy-of-{}", original_name)
}
