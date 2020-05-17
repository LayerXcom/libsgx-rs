use crate::std::{
    vec::Vec,
    sync::Arc,
    net::TcpStream,
    string::ToString,
};
use crate::{
    transport::{Message, TlsTransport},
    request::{RequestBuilder, Request},
    response::Response,
    into_url::IntoUrl,
};
use anyhow::{Result, anyhow};
use http::{
    Method,
    header::{HeaderMap, HeaderValue, ACCEPT},
};

#[derive(Clone)]
pub struct Client {
    config: Config,
}

impl Client {
    pub fn new() -> Client {
        Client {
            config: Config {
                tls_config: rustls::ClientConfig::default(),
            }
        }
    }

    // pub fn builder() -> ClientBuilder {
    //     ClientBuilder::new()
    // }

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
        use webpki::DNSNameRef;

        let url = req.url();
        let host = url.host().ok_or(anyhow!("no host in url"))?.to_string();
        let dnsname = DNSNameRef::try_from_ascii_str(&host)?;
        let sess = rustls::ClientSession::new(&self.config_arc(), dnsname);
        let stream = TcpStream::connect(url.as_str())?;
        let tls_stream = rustls::StreamOwned::new(sess, stream);
        let mut transport = TlsTransport::new(tls_stream);

        let response = transport.send(&req.into_url())?;

        Ok(Response { inner: response })
    }

    pub fn config_arc(&self) -> Arc<rustls::ClientConfig> {
        Arc::new(self.config.tls_config.clone())
    }
}

#[derive(Clone)]
struct Config {
    tls_config: rustls::ClientConfig,
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
