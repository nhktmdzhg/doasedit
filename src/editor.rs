use crate::error::{
    doas_cat_permission_denied, doas_unavailable, editor_error, invalid_editor,
    no_editor_specified, user_abort, Result,
};
use crate::utils::{create_copy_filename, get_filename, read_user_input};
use std::env;
use std::fs;

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Get the editor command from environment variables or default to vi
pub fn get_editor_command() -> Result<String> {
    // Try environment variables in order: DOAS_EDITOR, VISUAL, EDITOR
    for var in ["DOAS_EDITOR", "VISUAL", "EDITOR"] {
        if let Ok(val) = env::var(var) {
            if !val.is_empty() {
                // Check if command exists
                if Command::new(&val)
                    .arg("--version")
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .is_ok()
                {
                    return Ok(val);
                } else {
                    return Err(invalid_editor(&val));
                }
            }
        }
    }

    // Default to vi if available
    if Command::new("vi")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
    {
        return Ok("vi".to_string());
    }

    Err(no_editor_specified())
}

/// Open a file with the specified editor
pub fn open_file_with_editor(file_path: &Path, editor: &str) -> Result<()> {
    let status = Command::new(editor)
        .arg(file_path)
        .status()
        .map_err(|e| invalid_editor(&e.to_string()))?;

    if status.success() {
        Ok(())
    } else {
        Err(editor_error())
    }
}

/// Validate doas configuration file, allowing the user to fix any errors
pub fn validate_doas_config(tmp_file_path: &Path, editor_cmd: &str) -> Result<()> {
    loop {
        let output = Command::new("doas")
            .arg("-C")
            .arg(tmp_file_path)
            .output()
            .map_err(|_| doas_unavailable())?;

        if output.status.success() {
            break;
        }

        eprint!(
            "doasedit: Replacing '{}' would introduce the above error and break doas.\n",
            tmp_file_path.display()
        );

        let input = read_user_input("(E)dit again, (O)verwrite anyway, (A)bort: [E/o/a]? ")?;

        match input.trim().to_lowercase().as_str() {
            "o" => break,
            "a" => return Err(user_abort()),
            _ => {
                open_file_with_editor(tmp_file_path, editor_cmd)?;
            }
        }
    }

    Ok(())
}

/// Copy a file to a temporary location with secure permissions
pub fn create_secure_temp_copy(file_path: &Path, temp_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    let filename = get_filename(file_path)?;
    let copy_name = create_copy_filename(&filename);

    // Create temporary files paths
    let tmp_file_path = temp_dir.join(&filename);
    let tmp_copy_path = temp_dir.join(&copy_name);

    // Create the files
    fs::write(&tmp_file_path, "")?;
    fs::write(&tmp_copy_path, "")?;

    // Set secure permissions (0600)
    let mut perms = fs::metadata(&tmp_file_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&tmp_file_path, perms)?;

    let mut perms = fs::metadata(&tmp_copy_path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&tmp_copy_path, perms)?;

    Ok((tmp_file_path, tmp_copy_path))
}

/// Read the original file content into the temporary file
pub fn copy_original_content(
    original_path: &Path,
    temp_file_path: &Path,
    use_doas: bool,
) -> Result<()> {
    if use_doas {
        let output = Command::new("doas")
            .arg("cat")
            .arg(original_path)
            .output()
            .map_err(|_| doas_cat_permission_denied())?;

        if !output.status.success() {
            return Err(doas_cat_permission_denied());
        }

        fs::write(temp_file_path, output.stdout)?;
    } else {
        fs::copy(original_path, temp_file_path)?;
    }

    Ok(())
}

/// Create a copy of the temporary file for comparison
pub fn create_comparison_copy(tmp_file_path: &Path, tmp_copy_path: &Path) -> Result<()> {
    fs::copy(tmp_file_path, tmp_copy_path)?;
    Ok(())
}
