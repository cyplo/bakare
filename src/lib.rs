extern crate core;
extern crate crypto;
#[cfg(test)]
extern crate dir_diff;
#[cfg(test)]
extern crate tempfile;
extern crate walkdir;

pub mod backup;
pub mod restore;

mod storage;
