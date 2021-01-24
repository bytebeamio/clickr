mod client;
mod insert;
mod inserter;
mod response;

use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/o failure = {0}")]
    IO(io::Error),
    #[error("Hyper error = {0}")]
    Hyper(#[from] hyper::Error),
    #[error("bad response: {0}")]
    BadResponse(String),
    #[error("Custrom error: {0}")]
    Custom(String),
}

pub use client::Client;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
