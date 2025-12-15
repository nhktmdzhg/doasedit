mod editor;
mod error;
mod file_handler;
mod utils;

use clap::Arg;
use error::DoaseditError;
use file_handler::process_file;
use nix::unistd::geteuid;
use std::process::Command;
use tempfile::tempdir;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let matches = clap::Command::new("doasedit")
        .version("1.0.0")
        .about(
            "A Rust implementation of doasedit - edit files as root using an unprivileged editor",
        )
        .arg(
            Arg::new("files")
                .help("Files to edit")
                .required(true)
                .num_args(1..),
        )
        .get_matches();

    // Check if running as root
    if geteuid().is_root() {
        return Err(Box::new(DoaseditError::RootUserNotAllowed));
    }

    // Check if doas is available
    if !Command::new("doas")
        .arg("dd")
        .arg("status=none")
        .arg("count=0")
        .arg("of=/dev/null")
        .output()?
        .status
        .success()
    {
        return Err(Box::new(DoaseditError::DoasUnavailable));
    }

    // Determine editor command
    let editor =
        editor::get_editor_command().map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Create temporary directory
    let tmp_dir = tempdir()?;

    let mut _exit_code = 1;

    if let Some(files) = matches.get_many::<String>("files") {
        for file_path in files {
            match process_file(file_path, &editor, tmp_dir.path()) {
                Ok(_) => {
                    _exit_code = 0;
                }
                Err(e) => {
                    eprintln!("doasedit: {}", e);
                    continue;
                }
            }
        }
    }

    Ok(())
}
