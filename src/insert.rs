use std::{future::Future, mem, panic};

use crate::Error;

use crate::client::Client;
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
    pub(crate) fn new(client: &Client, table: &str) -> Result<Self, Error> {
        let mut url = Url::parse(&client.url).expect("TODO");
        let mut pairs = url.query_pairs_mut();
        pairs.clear();

        if let Some(database) = &client.database {
            pairs.append_pair("database", database);
        }

        todo!()
    }

    pub fn write<'a>(
        &'a mut self,
        payload: Bytes,
    ) -> impl Future<Output = Result<(), Error>> + 'a + Send {
        self.buffer.extend_from_slice(&payload[..]);

        async move {
            self.send_chunk_if_exceeds(MIN_CHUNK_SIZE).await?;
            Ok(())
        }
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
