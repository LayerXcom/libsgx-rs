use crate::std::vec::Vec;
use crate::{
    transport::{Message, TlsTransport},
    request::{RequestBuilder, Request},
    response::Response,
    into_url::IntoUrl,
};
use anyhow::Result;
use http::{
    Method,
    header::{HeaderMap, HeaderValue, ACCEPT},
};

#[derive(Clone)]
pub struct Client {

}

impl Client {
    pub fn new() -> Client {
        ClientBuilder::new().build().expect("Client::new()")
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        let req = url.into_url().map(move |url| Request::new(method, url));
        RequestBuilder::new(self.clone(), req)
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    pub fn execute(&self, req: Request) -> Result<Response> {

        unimplemented!();
    }
}

/// A `ClientBuilder` can be used to create a `Client` with  custom configuration.
pub struct ClientBuilder {
    config: Config,
}

impl ClientBuilder {
    pub fn new() -> Self {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::with_capacity(2);
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

        ClientBuilder {
            config: Config {
                config: rustls::ClientConfig::default(),
                headers,
            }
        }
    }

    pub fn build(self) -> Result<Client> {
        let config = self.config;

        unimplemented!();
    }
}

struct Config {
    config: rustls::ClientConfig,
    headers: HeaderMap, // default headers
}


pub trait ClientTransport {
    fn send(&mut self, req: &[u8]) -> Result<Vec<u8>>;
}

impl<S: rustls::Session> ClientTransport for TlsTransport<S> {
    fn send(&mut self, req: &[u8]) -> Result<Vec<u8>> {
        let mut msg = Message::new(&mut self.stream);
        msg.write(req)?;
        msg.read()
    }
}
