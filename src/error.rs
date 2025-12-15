use thiserror::Error;

#[derive(Error, Debug)]
pub enum DoaseditError {
    #[error("using this program as root is not permitted")]
    RootUserNotAllowed,
    #[error("unable to run 'doas'")]
    DoasUnavailable,
    #[error("no editor specified")]
    NoEditorSpecified,
    #[error("invalid editor command: '{0}'")]
    InvalidEditor(String),
    #[error("{0}: cannot edit directories")]
    CannotEditDirectory(String),
    #[error("{0}: not a regular file")]
    NotRegularFile(String),
    #[error("{0}: editing your own files is not permitted")]
    CannotEditOwnFile(String),
    #[error("{0}: creating files in your own directory is not permitted")]
    CannotCreateFileInOwnDir(String),
    #[error("{0}: creating files in a user-writable directory is not permitted")]
    CannotCreateFileInWritableDir(String),
    #[error("{0}: no such directory")]
    NoDirectoryExists(String),
    #[error("{0}: editing user-readable and -writable files is not permitted")]
    CannotEditReadableWritableFile(String),
    #[error("you are not permitted to call 'doas cat'")]
    DoasCatPermissionDenied,
    #[error("3 incorrect password attempts")]
    ThreeIncorrectPasswordAttempts,
    #[error("doas validation error: {0}")]
    DoasValidationError(String),
    #[error("interrupted")]
    Interrupted,
    #[error("editor exited with non-zero status")]
    EditorError,
    #[error("aborted by user")]
    UserAbort,
}

impl From<std::io::Error> for DoaseditError {
    fn from(err: std::io::Error) -> Self {
        DoaseditError::DoasValidationError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, DoaseditError>;
