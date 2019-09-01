use std::io;

use failure::Fail;
use std::path::StripPrefixError;

#[derive(Debug, Fail)]
pub enum BakareError {
    #[fail(display = "io error")]
    IOError(Option<io::Error>),
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
    IndexLoadingError,
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

impl From<rmp_serde::decode::Error> for BakareError {
    fn from(_: rmp_serde::decode::Error) -> Self {
        BakareError::IndexLoadingError
    }
}
