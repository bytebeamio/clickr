use std::io;

mod client;
mod db;
mod options;

pub use db::{Database, Inserter, Type};
pub use options::ClientOptions;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/o failure = {0}")]
    IO(io::Error),
    #[error("Ureq error = {0}")]
    Hyper(#[from] ureq::Error),
    #[error("bad response: {0}")]
    BadResponse(String),
    #[error("Custrom error: {0}")]
    Custom(String),
    #[error("invalid params: {0}")]
    InvalidParams(#[source] Box<dyn std::error::Error + Send + Sync>),
}

#[non_exhaustive]
pub enum Compression {
    None,
    #[cfg(feature = "lz4")]
    Lz4,
    #[cfg(feature = "gzip")]
    Gzip,
    #[cfg(feature = "zlib")]
    Zlib,
    #[cfg(feature = "brotli")]
    Brotli,
}

impl Default for Compression {
    #[cfg(feature = "lz4")]
    #[inline]
    fn default() -> Self {
        Compression::Lz4
    }

    #[cfg(not(feature = "lz4"))]
    #[inline]
    fn default() -> Self {
        Compression::None
    }
}
