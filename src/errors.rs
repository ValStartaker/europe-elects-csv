use thiserror::Error;

#[derive(Error, Debug)]
pub enum PollTableError {
    #[error("Failed to create ReaderBuilder from specified path")]
    ReaderBuilderError(#[from] csv::Error)
}