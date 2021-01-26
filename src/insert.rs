use std::{mem, panic};

use crate::Error;

use crate::client::Client;
use crate::response::{Compression, Response};
use bytes::{Bytes, BytesMut};
use hyper::{self, body, Body, Request};
use tokio::task::JoinHandle;
use url::Url;

const BUFFER_SIZE: usize = 128 * 1024;
const MIN_CHUNK_SIZE: usize = BUFFER_SIZE - 1024;

pub struct Insert {
    buffer: BytesMut,
    sender: Option<body::Sender>,
    handle: JoinHandle<Result<(), Error>>,
}

impl Insert {
    pub(crate) fn new(client: &Client, table: &str, columns: Vec<String>) -> Result<Self, Error> {
        let mut url = Url::parse(&client.url).expect("TODO");
        let mut pairs = url.query_pairs_mut();
        pairs.clear();

        if let Some(database) = &client.database {
            pairs.append_pair("database", database);
        }

        let fields = columns.join(",");
        let query = format!("INSERT INTO {}({}) FORMAT JSONEachRow", table, fields);
        pairs.append_pair("query", &query);
        drop(pairs);

        let mut builder = Request::post(url.as_str());

        if let Some(user) = &client.user {
            builder = builder.header("X-ClickHouse-User", user);
        }

        if let Some(password) = &client.password {
            builder = builder.header("X-ClickHouse-Key", password);
        }

        let (sender, body) = Body::channel();

        let request = builder
            .body(body)
            .map_err(|err| Error::InvalidParams(Box::new(err)))?;

        let future = client.client.request(request);
        let handle = tokio::spawn(async move {
            // TODO: should we read the body?
            let _ = Response::new(future, Compression::None).resolve().await?;
            Ok(())
        });

        Ok(Insert {
            buffer: BytesMut::with_capacity(BUFFER_SIZE),
            sender: Some(sender),
            handle,
        })
    }

    pub async fn write_bytes(&mut self, payload: Bytes) -> Result<(), Error> {
        self.buffer.extend_from_slice(&payload[..]);
        self.send_chunk_if_exceeds(MIN_CHUNK_SIZE).await?;
        Ok(())
    }

    pub async fn write_slice(&mut self, payload: &[u8]) -> Result<(), Error> {
        self.buffer.extend_from_slice(payload);
        self.send_chunk_if_exceeds(MIN_CHUNK_SIZE).await?;
        Ok(())
    }

    pub async fn end(mut self) -> Result<(), Error> {
        self.send_chunk_if_exceeds(1).await?;
        drop(self.sender.take());

        match (&mut self.handle).await {
            Ok(res) => res,
            Err(err) if err.is_panic() => panic::resume_unwind(err.into_panic()),
            Err(err) => Err(Error::Custom(format!("unexpected error: {}", err))),
        }
    }

    async fn send_chunk_if_exceeds(&mut self, threshold: usize) -> Result<(), Error> {
        if self.buffer.len() >= threshold {
            // Hyper uses non-trivial and inefficient (see benches) schema of buffering chunks.
            // It's difficult to determine when allocations occur.
            // So, instead we control it manually here and rely on the system allocator.
            let chunk = mem::replace(&mut self.buffer, BytesMut::with_capacity(BUFFER_SIZE));

            if let Some(sender) = &mut self.sender {
                sender.send_data(chunk.freeze()).await?;
            }
        }

        Ok(())
    }
}

impl Drop for Insert {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            sender.abort();
        }
    }
}
