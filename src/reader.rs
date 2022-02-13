use crate::{ClientOptions, Error};
use bytes::{Bytes, BytesMut};
use ureq::{Request, Response};
use url::Url;

const BUFFER_SIZE: usize = 128 * 1024;

pub struct Reader {
    request: Request,
    buffer: BytesMut,
}

impl Reader {
    pub fn new(options: ClientOptions) -> Reader {
        let mut url = Url::parse(&options.url).expect("TODO");
        let query = format!("select * Motor_Status_1");

        url.query_pairs_mut()
            .append_pair("database", &options.database);

        url.query_pairs_mut().append_pair("query", &query);

        let mut request = ureq::post(url.as_str());

        if let Some(user) = &options.user {
            request = request.set("X-ClickHouse-User", user);
        }

        if let Some(password) = &options.password {
            request = request.set("X-ClickHouse-Key", password);
        }

        Reader {
            request,
            buffer: BytesMut::with_capacity(BUFFER_SIZE),
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn write_bytes(&mut self, payload: Bytes) -> Result<(), Error> {
        self.buffer.extend_from_slice(&payload[..]);
        Ok(())
    }

    pub fn write_slice(&mut self, payload: &[u8]) -> Result<(), Error> {
        self.buffer.extend_from_slice(payload);
        Ok(())
    }

    pub fn end(&mut self) -> Result<Response, Error> {
        let request = self.request.clone();
        let response = request.send_bytes(&[])?;
        self.buffer.clear();
        Ok(response)
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}
