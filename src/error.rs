use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(ThisError, Debug, PartialEq)]
pub enum Error {
    #[error("{0}")]
    IO(#[source] IOError),

    #[error("invalid signature: '{0}' '{1}' '{2}'")]
    Signature(char, char, char),
    #[error("invalid previous tag size: {0}")]
    PreviousTagSize(u32),
}

#[derive(Debug, ThisError)]
#[error("io error: {0}")]
pub struct IOError(std::io::Error);

impl PartialEq for IOError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(IOError(e))
    }
}