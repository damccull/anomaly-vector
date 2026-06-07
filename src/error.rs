use thiserror::Error;

#[derive(Error, Debug)]
pub enum NmsFileReadError {
    #[error("Invalid DOS header: {0}")]
    InvalidDosHeader(&'static str),
    #[error("Invalid PE header: {0}")]
    InvalidPeHeader(&'static str),
    #[error("Invalid COFFS header: {0}")]
    InvalidCoffsHeader(&'static str),
    #[error("Invalid or malformed optional header")]
    InvalidOptionalHeader,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
