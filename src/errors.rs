use std::io;

use jurisdiction::Jurisdiction;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PollTableTryFromPathError {
    #[error("Failed to create ReaderBuilder from specified path")]
    ReaderBuilderError(#[from] csv::Error),
    #[error("Failed to get jurisdiction while trying to create PollTable from path")]
    InvalidJurisdictionError(#[from] GetJurisdictionError)
}

#[derive(Error, Debug)]
pub enum GetJurisdictionError {
    #[error("Specified file is not a .csv")]
    NotCsvError,
    #[error("Specified path is not a valid OsStr")]
    InvalidPathError,
    #[error("Could not create Jurisdiction from path")]
    JurisdictionCreateError(#[from] anyhow::Error)
}

#[derive(Error, Debug)]
pub enum GetDateRangeError {
    #[error("Failed to create ReaderBuilder from specified path")]
    ReaderBuilderError(#[from] csv::Error)
}