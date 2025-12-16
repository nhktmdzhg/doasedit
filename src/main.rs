mod editor;
mod error;
mod file_handler;
mod utils;

use clap::Arg;
use error::{doas_unavailable, root_user_not_allowed};
use file_handler::process_file;
use nix::unistd::geteuid;
use std::process::Command;
use tempfile::tempdir;

fn main() {
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
        eprintln!("doasedit: {}", root_user_not_allowed().to_string());
        std::process::exit(1);
    }

    // Check if doas is available
    match Command::new("doas")
        .arg("dd")
        .arg("status=none")
        .arg("count=0")
        .arg("of=/dev/null")
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                eprintln!("doasedit: {}", doas_unavailable().to_string());
                eprintln!(
                    "  Command failed with exit code: {:?}",
                    output.status.code()
                );
                if !output.stderr.is_empty() {
                    eprintln!("  stderr: {}", String::from_utf8_lossy(&output.stderr));
                }
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("doasedit: Error running 'doas' command: {}", e);
            std::process::exit(1);
        }
    }

    // Determine editor command
    let editor = match editor::get_editor_command() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("doasedit: Error getting editor command: {}", e);
            std::process::exit(1);
        }
    };

    // Create temporary directory
    let tmp_dir = match tempdir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("doasedit: Error creating temporary directory: {}", e);
            std::process::exit(1);
        }
    };

    let mut _exit_code = 1;

    if let Some(files) = matches.get_many::<String>("files") {
        for file_path in files {
            match process_file(file_path, &editor, tmp_dir.path()) {
                Ok(_) => {
                    _exit_code = 0;
                }
                Err(e) => {
                    eprintln!("doasedit: {}", e.to_string());
                    std::process::exit(1);
                }
            }
        }
    }
}
