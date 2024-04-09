use thiserror::Error;

#[derive(Error, Debug)]
pub enum PollTableTryFromPathError {
    #[error("Failed to create ReaderBuilder from specified path")]
    ReaderBuilderError(#[from] csv::Error),
    #[error("Specified file is not a .csv")]
    NotCsvError,
    #[error("Specified path is not a valid OsStr")]
    InvalidPathError,
    #[error("Filename does not match a valid Europe Elects jurisdiction")]
    InvalidJurisdictionError
}

#[derive(Error, Debug)]
pub enum PollTableFromStrError {
    #[error("Failed to create ReaderBuilder from specified &str")]
    ReaderBuilderError(#[from] csv::Error),
    #[error("Filename does not match a valid Europe Elects jurisdiction")]
    InvalidJurisdictionError
}

#[derive(Error, Debug)]
pub enum RawPollTableFromStrError {
    #[error("Failed to create ReaderBuilder from specified &str")]
    ReaderBuilderError(#[from] csv::Error),
}