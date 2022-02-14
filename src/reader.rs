use crate::{ClientOptions, Error};
use ureq::{Request, Response};
use url::Url;

pub struct Reader {
    request: Request
}

impl Reader {
    pub fn new(options: ClientOptions) -> Reader {
        let mut url = Url::parse(&options.url).expect("TODO");
        url.query_pairs_mut()
            .append_pair("database", &options.database);

        let mut request = ureq::post(url.as_str());

        if let Some(user) = &options.user {
            request = request.set("X-ClickHouse-User", user);
        }

        if let Some(password) = &options.password {
            request = request.set("X-ClickHouse-Key", password);
        }

        request = request.set("X-ClickHouse-Format", "JSON");
        
        Reader {
            request
        }
    }

    pub fn query(&self, query: &[u8]) -> Result<Response, Error> {
        let request = self.request.clone();
        let response = request.send_bytes(query)?;
        Ok(response)
    }

}
