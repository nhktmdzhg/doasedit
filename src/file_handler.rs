use crate::error::{DoaseditError, Result};
use crate::utils::{
    files_match, get_file_metadata_with_doas, get_filename, get_parent_directory,
    is_dir_owned_by_user, is_dir_writable_by_user, is_directory_path, is_doas_config_file,
    is_file_owned_by_user, is_file_writable_by_user,
};
use nix::unistd::getuid;
use std::fs;

use std::path::Path;
use std::process::Command;

/// Information about a file's status
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub exists: bool,
    pub readable: bool,
    pub writable: bool,
    pub is_directory: bool,
    pub is_owned_by_user: bool,
}

/// Check the status of a file and its permissions
pub fn check_file_status(path: &Path) -> Result<FileInfo> {
    let uid = getuid().as_raw();

    // First try without doas
    if path.exists() {
        let metadata = fs::metadata(path)?;
        let is_directory = metadata.is_dir();

        if is_directory {
            return Ok(FileInfo {
                exists: true,
                readable: false,
                writable: false,
                is_directory: true,
                is_owned_by_user: false,
            });
        }

        let is_owned = is_file_owned_by_user(path)?;
        let readable = metadata.permissions().readonly() == false;
        let writable = is_file_writable_by_user(path)?;

        return Ok(FileInfo {
            exists: true,
            readable,
            writable,
            is_directory: false,
            is_owned_by_user: is_owned,
        });
    }

    // Try with doas
    let check_existence = Command::new("doas")
        .arg("test")
        .arg("-e")
        .arg(path)
        .output()
        .map_err(|_| DoaseditError::DoasUnavailable)?;

    if check_existence.status.success() {
        let check_is_file = Command::new("doas")
            .arg("test")
            .arg("-f")
            .arg(path)
            .output()
            .map_err(|_| DoaseditError::DoasUnavailable)?;

        if !check_is_file.status.success() {
            return Ok(FileInfo {
                exists: true,
                readable: false,
                writable: false,
                is_directory: true,
                is_owned_by_user: false,
            });
        }

        // Get metadata with doas
        let (file_uid, mode) = get_file_metadata_with_doas(path)?;
        let writable = mode & 0o200 != 0; // User write bit
        let is_owned = file_uid == uid;

        return Ok(FileInfo {
            exists: true,
            readable: true, // Assuming readable if doas can read it
            writable,
            is_directory: false,
            is_owned_by_user: is_owned,
        });
    }

    // File doesn't exist, check directory
    let dir_path = get_parent_directory(path);

    if dir_path.exists() {
        // Check if directory is owned by user
        if is_dir_owned_by_user(&dir_path)? {
            return Err(DoaseditError::CannotCreateFileInOwnDir(path.display().to_string()).into());
        }

        // Check if directory is writable by user
        if is_dir_writable_by_user(&dir_path)? {
            return Err(
                DoaseditError::CannotCreateFileInWritableDir(path.display().to_string()).into(),
            );
        }
    } else {
        // Try with doas
        let check_dir_existence = Command::new("doas")
            .arg("test")
            .arg("-d")
            .arg(&dir_path)
            .output()
            .map_err(|_| DoaseditError::DoasUnavailable)?;

        if !check_dir_existence.status.success() {
            return Err(DoaseditError::NoDirectoryExists(dir_path.display().to_string()).into());
        }
    }

    Ok(FileInfo {
        exists: false,
        readable: false,
        writable: false,
        is_directory: false,
        is_owned_by_user: false,
    })
}

/// Process a file: create temp files, open editor, validate changes, and write back
pub fn process_file(file_path: &str, editor: &str, tmp_dir: &Path) -> Result<()> {
    use crate::editor::{
        copy_original_content, create_comparison_copy, create_secure_temp_copy,
        open_file_with_editor, validate_doas_config,
    };

    // Check if path is a directory (ends with /)
    if is_directory_path(file_path) {
        return Err(DoaseditError::CannotEditDirectory(file_path.to_string()).into());
    }

    let path = Path::new(file_path);
    let _filename = get_filename(path)?;

    // Check file existence and permissions
    let file_info = check_file_status(path)?;

    // Create temporary files using the editor module
    let (tmp_file_path, tmp_copy_path) = create_secure_temp_copy(path, tmp_dir)?;

    // If file exists, copy its content to temporary file
    if file_info.exists {
        // Check if user is not the owner of the file
        if file_info.is_owned_by_user {
            return Err(DoaseditError::CannotEditOwnFile(file_path.to_string()).into());
        }

        // Check if file is not a directory
        if file_info.is_directory {
            return Err(DoaseditError::NotRegularFile(file_path.to_string()).into());
        }

        // Check if file is not both readable and writable by user
        if file_info.readable && file_info.writable {
            return Err(
                DoaseditError::CannotEditReadableWritableFile(file_path.to_string()).into(),
            );
        }

        let use_doas = !file_info.readable;
        copy_original_content(path, &tmp_file_path, use_doas)?;

        // Create a copy for comparison
        create_comparison_copy(&tmp_file_path, &tmp_copy_path)?;
    }

    // Open the file with editor
    open_file_with_editor(&tmp_file_path, editor)?;

    // Check if this is a doas config file and validate if needed
    if is_doas_config_file(file_path) {
        validate_doas_config(&tmp_file_path, editor)?;
    }

    // Compare files and write back if changed
    if !files_match(&tmp_file_path, &tmp_copy_path)? {
        write_file_back(&tmp_file_path, path, file_info.writable)?;
    } else {
        println!("doasedit: {}: unchanged", file_path);
    }

    Ok(())
}

/// Write the modified content back to the original file
pub fn write_file_back(tmp_file_path: &Path, original_path: &Path, writable: bool) -> Result<()> {
    if writable {
        fs::copy(tmp_file_path, original_path)?;
    } else {
        // Try with doas (with retry for password)
        let mut success = false;
        for attempt in 0..3 {
            let output = Command::new("doas")
                .arg("dd")
                .arg("status=none")
                .arg(format!("if={}", tmp_file_path.display()))
                .arg(format!("of={}", original_path.display()))
                .output()
                .map_err(|_| DoaseditError::DoasUnavailable)?;

            if output.status.success() {
                success = true;
                break;
            }

            // If this is the third attempt, return error
            if attempt == 2 {
                return Err(DoaseditError::ThreeIncorrectPasswordAttempts);
            }
        }

        if !success {
            return Err(DoaseditError::ThreeIncorrectPasswordAttempts);
        }
    }

    Ok(())
}
