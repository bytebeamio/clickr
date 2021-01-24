use std::{
    pin::Pin,
    task::{Context, Poll},
};

use hyper::{body, client::ResponseFuture, Body, StatusCode};
use futures::stream::Stream;

use crate::{
    Error
};

use bytes::Bytes;

#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Compression {
    None,
}

impl Default for Compression {
    fn default() -> Self {
        Compression::None
    }
}

pub enum Response {
    Waiting(ResponseFuture, Compression),
    Loading(Chunks),
}

impl Response {
    pub fn new(future: ResponseFuture, compression: Compression) -> Self {
        Self::Waiting(future, compression)
    }

    pub async fn resolve(&mut self) -> Result<&mut Chunks, Error> {
        if let Self::Waiting(response, compression) = self {
            let response = response.await?;

            if response.status() != StatusCode::OK {
                let bytes = body::to_bytes(response.into_body()).await?;
                let reason = String::from_utf8_lossy(&bytes).trim().into();

                return Err(Error::BadResponse(reason));
            }

            let body = response.into_body();
            let chunks = match compression {
                Compression::None => Inner::Plain(body),
            };

            *self = Self::Loading(Chunks(chunks));
        }

        match self {
            Self::Waiting(..) => unreachable!(),
            Self::Loading(chunks) => Ok(chunks),
        }
    }
}

pub struct Chunks(Inner);

enum Inner {
    Plain(Body),
    Empty,
}

impl Stream for Chunks {
    type Item = Result<Bytes, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use Inner::*;
        let res = match self.0 {
            Plain(ref mut inner) => map_poll_err(Pin::new(inner).poll_next(cx), Into::into),
            Empty => Poll::Ready(None),
        };

        if let Poll::Ready(None) = res {
            self.0 = Inner::Empty;
        }

        res
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        use Inner::*;

        match &self.0 {
            Plain(inner) => inner.size_hint(),
            Empty => (0, Some(0)),
        }
    }
}

// XXX: https://github.com/rust-lang/rust/issues/63514
fn map_poll_err<T, E, E2>(
    poll: Poll<Option<Result<T, E>>>,
    f: impl FnOnce(E) -> E2,
) -> Poll<Option<Result<T, E2>>> {
    match poll {
        Poll::Ready(Some(Ok(val))) => Poll::Ready(Some(Ok(val))),
        Poll::Ready(Some(Err(err))) => Poll::Ready(Some(Err(f(err)))),
        Poll::Ready(None) => Poll::Ready(None),
        Poll::Pending => Poll::Pending,
    }
}
