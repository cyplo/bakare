use std::io;

use failure::Fail;
use std::path::StripPrefixError;

#[derive(Debug, Fail)]
pub enum BakareError {
    #[fail(display = "io error")]
    IOError(Option<io::Error>),
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

impl From<io::Error> for BakareError {
    fn from(e: io::Error) -> Self {
        BakareError::IOError(Some(e))
    }
}

impl From<walkdir::Error> for BakareError {
    fn from(e: walkdir::Error) -> Self {
        BakareError::IOError(e.into_io_error())
    }
}

impl From<StripPrefixError> for BakareError {
    fn from(_: StripPrefixError) -> Self {
        BakareError::IOError(None)
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
