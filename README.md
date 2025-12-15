# Doasedit

A Rust implementation of doasedit - edit files as root using an unprivileged editor.

## Description

`doasedit` is a security-focused tool that enables regular users to safely edit files requiring root privileges while using their preferred editor. Written in Rust for enhanced security and performance, it provides a secure environment for editing system configuration files without running the editor itself with elevated privileges.

## Features

- **Secure temporary file handling**: Creates temporary files with restricted permissions (0600)
- **Permission validation**: Prevents editing files owned by current user or in user-writable directories
- **Configuration validation**: Validates doas configuration files before installation
- **Multiple editor support**: Respects `DOAS_EDITOR`, `VISUAL`, and `EDITOR` environment variables
- **Password retry mechanism**: Supports up to 3 password attempts for privileged operations
- **Cross-platform compatibility**: Designed for Unix-like systems

## Installation

### From Source

```bash
git clone https://github.com/nhktmdzhg/doasedit.git
cd doasedit
cargo build --release
sudo cp target/release/doasedit /usr/local/bin/
```

### On Arch Linux

```bash
# Using makepkg
git clone https://github.com/nhktmdzhg/doasedit.git
cd doasedit
makepkg -si

# Or using an AUR helper
paru -S doasedit-nhk
```

## Usage

### Basic Usage

```bash
# Edit a system file
doasedit /etc/hosts

# Edit multiple files
doasedit /etc/fstab /etc/securetty

# Edit a doas configuration file
doasedit /etc/doas.conf
```

### Environment Variables

`doasedit` respects the following environment variables in order of precedence:

1. `DOAS_EDITOR` - Preferred editor specifically for doasedit
2. `VISUAL` - Preferred editor for interactive sessions
3. `EDITOR` - Default system editor

If none of these are set, `doasedit` defaults to `vi`.

### Examples

```bash
# Use nano as editor
DOAS_EDITOR=nano doasedit /etc/hosts

# Edit a doas configuration file
doasedit /etc/doas.conf
# If configuration contains errors, you'll be prompted to:
# (E)dit again, (O)verwrite anyway, (A)bort: [E/o/a]?

# Create a new file in /etc (requires doas permission)
doasedit /etc/new-config-file
```

## Security

`doasedit` implements several security measures to prevent privilege escalation:

- Cannot be executed as root user
- Will not edit files owned by current user
- Will not create files in directories owned by current user
- Will not create files in directories writable by non-root users
- Validates doas configuration files before installation

## Architecture

The application is organized into several modular components:

- **Editor module**: Handles editor detection and file editing
- **File handler module**: Manages file operations and permission checks
- **Utilities module**: Provides common helper functions
- **Error handling**: Comprehensive error management with proper error types

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- The Rust community for providing excellent tools and libraries
- The security community for best practices in privilege management
