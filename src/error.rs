use std::fmt;

#[derive(Debug)]
pub struct DoaseditError(String);

impl fmt::Display for DoaseditError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DoaseditError {}

pub type Result<T> = std::result::Result<T, DoaseditError>;

// Helper functions to create error messages
pub fn root_user_not_allowed() -> DoaseditError {
    DoaseditError("using this program as root is not permitted".to_string())
}

pub fn doas_unavailable() -> DoaseditError {
    DoaseditError("unable to run 'doas'".to_string())
}

pub fn no_editor_specified() -> DoaseditError {
    DoaseditError("no editor specified".to_string())
}

pub fn invalid_editor(editor: &str) -> DoaseditError {
    DoaseditError(format!("invalid editor command: '{}'", editor))
}

pub fn cannot_edit_directory(path: &str) -> DoaseditError {
    DoaseditError(format!("{}: cannot edit directories", path))
}

pub fn not_regular_file(path: &str) -> DoaseditError {
    DoaseditError(format!("{}: not a regular file", path))
}

pub fn cannot_edit_own_file(path: &str) -> DoaseditError {
    DoaseditError(format!("{}: editing your own files is not permitted", path))
}

pub fn cannot_create_file_in_own_dir(path: &str) -> DoaseditError {
    DoaseditError(format!(
        "{}: creating files in your own directory is not permitted",
        path
    ))
}

pub fn cannot_create_file_in_writable_dir(path: &str) -> DoaseditError {
    DoaseditError(format!(
        "{}: creating files in a user-writable directory is not permitted",
        path
    ))
}

pub fn no_directory_exists(path: &str) -> DoaseditError {
    DoaseditError(format!("{}: no such directory", path))
}

pub fn cannot_edit_readable_writable_file(path: &str) -> DoaseditError {
    DoaseditError(format!(
        "{}: editing user-readable and -writable files is not permitted",
        path
    ))
}

pub fn doas_cat_permission_denied() -> DoaseditError {
    DoaseditError("you are not permitted to call 'doas cat'".to_string())
}

pub fn three_incorrect_password_attempts() -> DoaseditError {
    DoaseditError("3 incorrect password attempts".to_string())
}

pub fn doas_validation_error(msg: &str) -> DoaseditError {
    DoaseditError(format!("doas validation error: {}", msg))
}

pub fn interrupted() -> DoaseditError {
    DoaseditError("interrupted".to_string())
}

pub fn editor_error() -> DoaseditError {
    DoaseditError("editor exited with non-zero status".to_string())
}

pub fn user_abort() -> DoaseditError {
    DoaseditError("aborted by user".to_string())
}

impl From<std::io::Error> for DoaseditError {
    fn from(err: std::io::Error) -> Self {
        DoaseditError(err.to_string())
    }
}
