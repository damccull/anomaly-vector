use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("I/O error while reading executable: {0}")]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}
