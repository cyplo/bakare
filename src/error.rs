use std::io;

use failure::Fail;

#[derive(Debug, Fail)]
pub enum BakareError {
    #[fail(display = "io error")]
    IOError,
    #[fail(display = "unknown source path")]
    UnknownSourcePathError,
}

impl From<io::Error> for BakareError {
    fn from(e: io::Error) -> Self {
        BakareError::IOError
    }
}

impl From<walkdir::Error> for BakareError {
    fn from(e: walkdir::Error) -> Self {
        BakareError::IOError
    }
}
