use std::sync::Arc;

use crate::{ClientOptions, Error};
use bytes::{Bytes, BytesMut};
use rustls::version::{TLS12, TLS13};
use ureq::{Request, Response};
use url::Url;

const BUFFER_SIZE: usize = 128 * 1024;

pub struct Inserter {
    request: Request,
    buffer: BytesMut,
}

impl Inserter {
    pub fn new(options: ClientOptions, table: &str) -> Inserter {
        let agent = match options.secure {
            true => {
                let mut root_store = rustls::RootCertStore::empty();

                let certs = rustls_native_certs::load_native_certs().expect("Could not load platform certs");
                for cert in certs {
                    // Repackage the certificate DER bytes.
                    let rustls_cert = rustls::Certificate(cert.0);
                    root_store
                        .add(&rustls_cert)
                        .expect("Failed to add native certificate too root store");
                }

                root_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(
                    |ta| {
                        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
                            ta.subject,
                            ta.spki,
                            ta.name_constraints,
                        )
                    },
                ));

                let protocol_versions = &[&TLS12, &TLS13];

                let tls_config = rustls::ClientConfig::builder()
                    .with_safe_default_cipher_suites()
                    .with_safe_default_kx_groups()
                    .with_protocol_versions(protocol_versions)
                    .unwrap()
                    .with_root_certificates(root_store)
                    .with_no_client_auth();

                ureq::builder().tls_config(Arc::new(tls_config)).build()
            }
            false => ureq::builder().build(),
        };

        let mut url = Url::parse(&options.url).expect("TODO");
        let query = format!("INSERT INTO {} FORMAT JSONEachRow", table);

        url.query_pairs_mut()
            .append_pair("database", &options.database);

        url.query_pairs_mut().append_pair("query", &query);

        let mut request = agent.post(url.as_str());
        // let mut request = ureq::post(url.as_str());

        if let Some(user) = &options.user {
            request = request.set("X-ClickHouse-User", user);
        }

        if let Some(password) = &options.password {
            request = request.set("X-ClickHouse-Key", password);
        }

        Inserter {
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
        let response = request.send_bytes(&self.buffer[..])?;
        self.buffer.clear();
        Ok(response)
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}
