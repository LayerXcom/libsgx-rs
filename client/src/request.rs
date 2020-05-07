use crate::std::{
    vec::Vec,
    time::Duration,
    convert::TryFrom,
};
use crate::{
    client::Client,
    response::Response,
};
use http::{
    Method,
    header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE},
};
use url::Url;
use serde::Serialize;
use anyhow::Result;

pub struct Request {
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    timeout: Option<Duration>,
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

/// A builder to construct the properties of a `Request`.
pub struct RequestBuilder {
    client: Client,
    request: Result<Request>,

}

impl RequestBuilder {
    pub(crate) fn new(client: Client, request: Result<Request>) -> Self {
        // TODO: Add configuration of basic auth
        RequestBuilder { client, request }
    }

    /// Add a `Header` to this Request.
    pub fn header<K, V>(mut self, key: K, value: V) -> Self
    where
        HeaderName: TryFrom<K>,
        HeaderValue: TryFrom<V>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        let mut error = None;
        if let Ok(ref mut req) = self.request {
            match <HeaderName as TryFrom<K>>::try_from(key) {
                Ok(key) => match <HeaderValue as TryFrom<V>>::try_from(value) {
                    Ok(value) => {
                        req.headers_mut().append(key, value);
                    }
                    Err(e) => error = Some(e.into()),
                },
                Err(e) => error = Some(e.into()),
            };
        }
        if let Some(err) = error {
            self.request = Err(err.into());
        }
        self
    }

    /// Sets the body to the JSON serialization of the passed value, and
    /// also sets the `Content-Type: application/json` header.
    pub fn json<T: Serialize + ?Sized>(mut self, json: &T) -> Self {
        let mut error = None;
        if let Ok(ref mut req) = self.request {
            match serde_json::to_vec(json) {
                Ok(body) => {
                    req.headers_mut()
                        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                    *req.body_mut() = Some(body.into());
                }
                Err(err) => error = Some(err),
            }
        }
        if let Some(err) = error {
            self.request = Err(err.into());
        }
        self
    }

    pub fn send(self) -> Result<Response> {
        self.client.execute(self.request?)
    }
}
