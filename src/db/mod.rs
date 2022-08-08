mod clickhouse;

use bytes::Bytes;
use clickhouse::Clickhouse;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use ureq::Response;

use crate::{ClientOptions, Error};

#[enum_dispatch(Database)]
pub trait Inserter {
    fn len(&self) -> usize;

    fn write_bytes(&mut self, payload: Bytes) -> Result<(), Error>;

    fn write_slice(&mut self, payload: &[u8]) -> Result<(), Error>;

    fn clear(&mut self);

    fn end(&mut self) -> Result<Response, Error>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Type {
    #[serde(rename = "clickhouse")]
    Clickhouse,
}

#[enum_dispatch]
pub enum Database {
    Clickhouse,
}

impl Database {
    pub fn new(db_type: &Type, options: ClientOptions, table: &str) -> Database {
        match db_type {
            Type::Clickhouse => Self::Clickhouse(Clickhouse::new(options, table)),
        }
    }
}
