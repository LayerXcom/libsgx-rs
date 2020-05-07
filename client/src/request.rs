use crate::std::{
    vec::Vec,
    time::Duration,
};
use crate::client::Client;
use crate::response::Response;
use http::{Method, header::HeaderMap};
use url::Url;
use anyhow::Result;

pub struct Request {
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    timeout: Option<Duration>,
}

/// A builder to construct the properties of a `Request`.
pub struct RequestBuilder {
    client: Client,
    request: Result<Request>,
}

impl Request {
    /// Constructs a new request.
    #[inline]
    pub fn new(method: Method, url: Url) -> Self {
        Request {
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            timeout: None,
        }
    }

    /// Get the method.
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Get a mutable reference to the method.
    #[inline]
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    /// Get the url.
    #[inline]
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Get a mutable reference to the url.
    #[inline]
    pub fn url_mut(&mut self) -> &mut Url {
        &mut self.url
    }

    /// Get the headers.
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a mutable reference to the headers.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Get the body.
    #[inline]
    pub fn body(&self) -> Option<&Vec<u8>> {
        self.body.as_ref()
    }

    /// Get a mutable reference to the body.
    #[inline]
    pub fn body_mut(&mut self) -> &mut Option<Vec<u8>> {
        &mut self.body
    }

    /// Get the timeout.
    #[inline]
    pub fn timeout(&self) -> Option<&Duration> {
        self.timeout.as_ref()
    }

    /// Get a mutable reference to the timeout.
    #[inline]
    pub fn timeout_mut(&mut self) -> &mut Option<Duration> {
        &mut self.timeout
    }
}

impl RequestBuilder {
    pub(crate) fn new(client: Client, request: Result<Request>) -> Self {
        // TODO: Add configuration of basic auth
        RequestBuilder { client, request }
    }

    pub fn header(mut self, ) -> Self {
        unimplemented!();
    }

    pub fn body(mut self) -> Self {
        unimplemented!();
    }

    pub fn send(self) -> Result<Response> {
        self.client.execute(self.request?)
    }
}
