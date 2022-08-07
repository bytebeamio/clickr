mod clickhouse;

use bytes::Bytes;
use clickhouse::ClickhouseInserter;
use serde::{Deserialize, Serialize};
use ureq::Response;

use crate::{ClientOptions, Error};

pub trait Inserter {
    fn new(db_type: &Type, options: ClientOptions, table: &str) -> Database;

    fn len(&self) -> usize;

    fn write_bytes(&mut self, payload: Bytes) -> Result<(), Error>;

    fn write_slice(&mut self, payload: &[u8]) -> Result<(), Error>;

    fn clear(&mut self);

    fn end(&mut self) -> Result<Response, Error>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Type {
    Clickhouse,
}

pub enum Database {
    Clickhouse(ClickhouseInserter),
}

impl Inserter for Database {
    fn new(db_type: &Type, options: ClientOptions, table: &str) -> Database {
        match db_type {
            Type::Clickhouse => Self::Clickhouse(ClickhouseInserter::new(options, table)),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Clickhouse(db) => db.len(),
        }
    }

    fn write_bytes(&mut self, payload: Bytes) -> Result<(), Error> {
        match self {
            Self::Clickhouse(db) => db.write_bytes(payload),
        }
    }

    fn write_slice(&mut self, payload: &[u8]) -> Result<(), Error> {
        match self {
            Self::Clickhouse(db) => db.write_slice(payload),
        }
    }

    fn clear(&mut self) {
        match self {
            Self::Clickhouse(db) => db.clear(),
        }
    }

    fn end(&mut self) -> Result<Response, Error> {
        match self {
            Self::Clickhouse(db) => db.end(),
        }
    }
}
