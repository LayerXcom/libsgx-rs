use crate::std::vec::Vec;
use crate::transport::{Message, TlsTransport};
use crate::request::RequestBuilder;
use crate::into_url::IntoUrl;
use anyhow::Result;
use http::Method;

pub struct Client {

}

pub struct ClientBuilder {

}

impl Client {
    pub fn new() -> Client {
        unimplemented!();
    }

    pub fn builder() -> ClientBuilder {
        unimplemented!();
    }

    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        unimplemented!();
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::POST, url)
    }
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
