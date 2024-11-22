use thiserror::Error;

#[derive(Error, Debug)]
pub enum FotosError {
    #[error("An I/O error occurred: {0}")]
    Io(#[from] std::io::Error),
    #[error("An SQLite error occurred: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("An migraion error occurred: {0}")]
    Migration(#[from] refinery::Error),

    #[error("An unknown error occurred")]
    Unknown,
    #[error("System time : {0}")]
    SystemTime(#[from] std::time::SystemTimeError),
}

pub type Result<T> = std::result::Result<T, FotosError>;
