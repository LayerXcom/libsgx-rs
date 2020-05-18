use crate::std::string::String;
use url::Url;
use anyhow::{Result, anyhow};

/// A trait to try to convert some type into a `Url`.
///
/// This trait is "sealed", such that only types within reqwest can
/// implement it. The reason is that it will eventually be deprecated
/// and removed, when `std::convert::TryFrom` is stabilized.
pub trait IntoUrl: PolyfillTryInto {}

impl<T: PolyfillTryInto> IntoUrl for T {}

pub trait PolyfillTryInto {
    // Besides parsing as a valid `Url`, the `Url` must be a valid
    // `http::Uri`, in that it makes sense to use in a network request.
    fn into_url(self) -> Result<Url>;
}

impl PolyfillTryInto for Url {
    fn into_url(self) -> Result<Url> {
        if self.has_host() {
            Ok(self)
        } else {
            Err(anyhow!("URL scheme is not allowed"))
        }
    }
}

impl<'a> PolyfillTryInto for &'a str {
    fn into_url(self) -> Result<Url> {
        Url::parse(self)?.into_url()
    }
}

impl<'a> PolyfillTryInto for &'a String {
    fn into_url(self) -> Result<Url> {
        (&**self).into_url()
    }
}
