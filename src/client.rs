use crate::response::Compression;
use crate::{insert, inserter, Error};
use hyper::client::HttpConnector;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Client {
    pub(crate) client: hyper::Client<HttpConnector>,

    pub(crate) url: String,
    pub(crate) database: Option<String>,
    pub(crate) user: Option<String>,
    pub(crate) password: Option<String>,
    compression: Compression,
    options: HashMap<String, String>,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            client: hyper::Client::new(),
            url: String::new(),
            database: None,
            user: None,
            password: None,
            compression: Compression::default(),
            options: HashMap::new(),
        }
    }
}

impl Client {
    // TODO: use `url` crate?
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.compression = compression;
        self
    }

    pub fn with_option(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.options.insert(name.into(), value.into());
        self
    }

    pub fn insert(&self, table: &str, columns: Vec<String>) -> Result<insert::Insert, Error> {
        insert::Insert::new(self, table, columns)
    }

    pub fn inserter(&self, table: &str, columns: Vec<String>) -> Result<inserter::Inserter, Error> {
        inserter::Inserter::new(self, table, columns)
    }
}
