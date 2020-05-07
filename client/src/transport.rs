use crate::std::{
    io::{Read, Write},
    vec::Vec,
    net::TcpStream,
};
use anyhow::{ensure, Result};

pub struct TlsTransport<S: rustls::Session> {
    pub stream: rustls::StreamOwned<S, TcpStream>,
}

impl<S: rustls::Session> TlsTransport<S> {
    pub fn new(stream: rustls::StreamOwned<S, TcpStream>) -> Self {
        TlsTransport { stream }
    }
}

pub struct Message<'a, T>
where
    T: Read + Write,
{
    transport: &'a mut T,
    max_frame_len: usize,
}

impl<T> Message<'_, T>
where
    T: Read + Write,
{
    pub fn new(transport: &'_ mut T) -> Message<'_, T> {
        Message {
            transport,
            max_frame_len: 8 * 1_024 * 1_024,
        }
    }

    pub fn read(&mut self) -> Result<Vec<u8>> {
        let mut buf = vec![];
        self.transport.read_exact(&mut buf)?;

        ensure!(buf.len() < self.max_frame_len, "Exceed max frame length");

        Ok(buf)
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<()>
    {
        self.transport.write_all(&buf)?;
        self.transport.flush()?;

        Ok(())
    }
}
