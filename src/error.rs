use std::{error, fmt};

#[derive(Debug, PartialEq)]
pub enum Error {
    Signature(SignatureError),
    PreviousTagSize(PreviousTagSizeError),
    IO(IOError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signature(e) => write!(f, "{}", e),
            Self::PreviousTagSize(e) => write!(f, "{}", e),
            Self::IO(e) => write!(f, "{}", e),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Signature(e) => e.source(),
            Self::PreviousTagSize(e) => e.source(),
            Self::IO(e) => Some(&e.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureError {
    pub signature: [u8; 3],
}

impl fmt::Display for SignatureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid signature {:?}", self.signature)
    }
}

impl error::Error for SignatureError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl From<&[u8]> for SignatureError {
    fn from(value: &[u8]) -> Self {
        SignatureError {
            signature: [value[0], value[1], value[2]],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviousTagSizeError {
    pub previous_tag: u8,
}

impl fmt::Display for PreviousTagSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid previous tag size {:?}", self.previous_tag)
    }
}

impl From<u8> for PreviousTagSizeError {
    fn from(value: u8) -> Self {
        PreviousTagSizeError {
            previous_tag: value,
        }
    }
}

#[derive(Debug)]
pub struct IOError(std::io::Error);

impl PartialEq for IOError {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind() == other.0.kind()
    }
}

impl fmt::Display for IOError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "io error: {}", self.0)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(IOError(e))
    }
}