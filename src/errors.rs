use thiserror::Error;

#[derive(Error, Debug)]
pub enum PollError {
    #[error("Failed to create ReaderBuilder from specified path")]
    ReaderBuilderError(#[from] csv::Error)
}