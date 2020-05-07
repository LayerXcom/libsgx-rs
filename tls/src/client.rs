use crate::std::vec::Vec;
use crate::transport::{Message, TlsTransport};
use anyhow::Result;

pub trait Client {
    fn send(&mut self, req: &[u8]) -> Result<Vec<u8>>;
}

impl<S: rustls::Session> Client for TlsTransport<S> {
    fn send(&mut self, req: &[u8]) -> Result<Vec<u8>> {
        let mut msg = Message::new(&mut self.stream);
        msg.write(req)?;
        msg.read()
    }
}
