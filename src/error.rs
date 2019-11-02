use std::io;
use std::path::{Path, StripPrefixError};

use failure::Fail;

#[derive(Debug, Fail)]
pub enum BakareError {
    #[fail(display = "io error")]
    IOError(Option<io::Error>, String),
    #[fail(display = "io error: globbing error")]
    IOGlobbingError(Option<glob::PatternError>),
    #[fail(display = "backup source same as repository")]
    SourceSameAsRepository,
    #[fail(display = "repository path is not absolute")]
    RepositoryPathNotAbsolute,
    #[fail(display = "path to store is not absolute")]
    PathToStoreNotAbsolute,
    #[fail(display = "directory used in place of a file")]
    DirectoryNotFile,
    #[fail(display = "corrupted repository - cannot find file")]
    CorruptedRepoNoFile,
    #[fail(display = "index loading error")]
    IndexLoadingError(Option<serde_cbor::Error>),
}

impl<T> From<(io::Error, T)> for BakareError
where
    T: AsRef<Path>,
{
    fn from((e, p): (io::Error, T)) -> Self {
        BakareError::IOError(Some(e), p.as_ref().to_string_lossy().to_string())
    }
}

impl From<walkdir::Error> for BakareError {
    fn from(e: walkdir::Error) -> Self {
        let io_error = e.into_io_error();
        BakareError::IOError(io_error, "walkdir".to_string())
    }
}

impl From<StripPrefixError> for BakareError {
    fn from(_: StripPrefixError) -> Self {
        BakareError::IOError(None, "strip prefix error".to_string())
    }
}

impl From<serde_cbor::Error> for BakareError {
    fn from(e: serde_cbor::Error) -> Self {
        BakareError::IndexLoadingError(Some(e))
    }
}

impl From<glob::PatternError> for BakareError {
    fn from(e: glob::PatternError) -> Self {
        BakareError::IOGlobbingError(Some(e))
    }
}

impl From<glob::GlobError> for BakareError {
    fn from(_: glob::GlobError) -> Self {
        BakareError::IOGlobbingError(None)
    }
}
