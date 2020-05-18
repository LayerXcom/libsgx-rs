use crate::std::{
    vec::Vec,
    time::Duration,
    convert::TryFrom,
    string::ToString,
};
use crate::{
    client::Client,
    response::Response,
};
use http::{
    Method,
    header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, CONTENT_TYPE, CONTENT_LENGTH},
};
use url::Url;
use serde::Serialize;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    url: Url,
    headers: HeaderMap,
    body: Option<Vec<u8>>,
    timeout: Option<Duration>,
}

impl Request {
    /// Constructs a new request.
    pub fn new(method: Method, url: Url) -> Self {
        let mut headers: HeaderMap<HeaderValue> = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

        Request {
            method,
            url,
            headers,
            body: None,
            timeout: None,
        }
    }

    pub fn into_url(self) -> Vec<u8> {
        let mut des = vec![];

        des.extend_from_slice(self.method().as_str().as_bytes());
        des.extend_from_slice(b" ");
        des.extend_from_slice(self.url().path().as_bytes());
        des.extend_from_slice(b" HTTP/1.1\r\nHOST: ");
        des.extend_from_slice(self.url().host().unwrap().to_string().as_bytes());
        des.extend_from_slice(b"\r\n");

        for (name, value) in self.headers() {
            des.extend_from_slice(name.as_str().as_bytes());
            des.extend_from_slice(b": ");
            des.extend_from_slice(value.as_bytes());
            des.extend_from_slice(b"\r\n");
        }

        des.extend_from_slice(b"Connection: close\r\n\r\n");
        if let Some(body) = self.body() {
            des.extend_from_slice(&body);
        }

        des
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
                    let len = body.len().to_string();
                    req.headers_mut()
                        .insert(CONTENT_LENGTH, HeaderValue::from_str(&len).unwrap());
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
