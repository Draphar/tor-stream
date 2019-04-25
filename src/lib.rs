//! Provides an interface for proxying network streams over the Tor network.
//!
//! See [setup] for information on creating a local Tor SOCKS5 proxy.
//!
//! # Usage
//!
//! If your Tor proxy is running on the default address `127.0.0.1:9050`,
//! you can use [`TorStream::connect()`]. If that is not the case,
//! you can specify your address in a call to [`TorStream::connect_with_address()`].
//!
//! ```
//! use tor_stream::TorStream;
//! use std::io::prelude::*;
//!
//! let mut stream = TorStream::connect("www.example.com:80").expect("Failed to connect");
//!
//! // The stream can be used like a normal TCP stream
//!
//! stream.write_all(b"GET / HTTP/1.1\r\nConnection: Close\r\nHost: www.example.com\r\n\r\n").expect("Failed to send request");
//!
//! // If you want the raw stream, call `unwrap`
//!
//! let mut stream = stream.unwrap();
//!
//! let mut buf = String::new();
//! stream.read_to_string(&mut buf).expect("Failed to read response");
//!
//! println!("Server response:\n{}", buf);
//! ```
//!
//! # Credits
//!
//! This crate is mostly a wrapper about Steven Fackler's [`socks`] crate.
//!
//! [setup]: setup/index.html
//! [`socks`]: https://crates.io/crates/socks
//! [`TorStream::connect()`]: struct.TorStream.html#method.connect
//! [`TorStream::connect_with_address()`]: struct.TorStream.html#method.connect_with_address

#![forbid(unsafe_code)]

#[macro_use]
extern crate lazy_static;
extern crate socks;

use socks::ToTargetAddr;

use socks::Socks5Stream;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream};
use std::ops::Deref;

lazy_static! {
    /// The default TOR socks5 proxy address, `127.0.0.1:9050`.
    /// The proxy can be configured in the SOCKS section of [`/etc/tor/torrc`].
    /// See the [TOR manual] for more information on `torrc`.
    ///
    /// [`/etc/tor/torrc`]: file:///etc/tor/torrc
    /// [TOR manual]: https://www.torproject.org/docs/tor-manual.html.en
    pub static ref TOR_PROXY: SocketAddr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
}

/// A stream proxied over the Tor network.
/// After connecting, it can be used like a normal [`TcpStream`].
///
/// [`TcpStream`]: https://doc.rust-lang.org/std/net/struct.TcpStream.html
pub struct TorStream(TcpStream);

impl TorStream {
    /// Connects to a destination address over the Tor network.
    ///
    /// # Requirements
    ///
    /// A Tor SOCKS5 proxy must be running at `127.0.0.1:9050`.
    /// See [setup] for more details on configuring a local proxy.
    ///
    /// If you want to use a different Tor address, use [`connect_with_address`].
    ///
    /// [setup]: setup/index.html
    /// [`connect_with_address`]: struct.TorStream.html#method.connect_with_address
    pub fn connect(destination: impl ToTargetAddr) -> io::Result<TorStream> {
        Socks5Stream::connect(TOR_PROXY.deref(), destination)
            .map(|stream| TorStream(stream.into_inner()))
    }

    /// Connects to a destination address over the Tor network.
    /// A Tor SOCKS5 proxy must be running at the `tor_proxy` address.
    pub fn connect_with_address(
        tor_proxy: SocketAddr,
        destination: impl ToTargetAddr,
    ) -> io::Result<TorStream> {
        Socks5Stream::connect(tor_proxy, destination).map(|stream| TorStream(stream.into_inner()))
    }

    /// Gets a reference to the underlying TCP stream.
    #[inline]
    pub fn get_ref(&self) -> &TcpStream {
        &self.0
    }

    /// Gets a mutable reference to the underlying TCP stream.
    #[inline]
    pub fn get_mut(&mut self) -> &mut TcpStream {
        &mut self.0
    }

    /// Unwraps the `TorStream`.
    #[inline]
    pub fn unwrap(self) -> TcpStream {
        self.0
    }
}

impl Read for TorStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for TorStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

/// # Instructions on setting up a local Tor proxy
///
/// First, install Tor locally.
/// On Linux, you can just install the `tor` package in most cases.
/// Visit the [Tor installation guide] for more specific instructions.
///
/// When Tor is installed, open Tor's configuration `torrc` (`/etc/tor/torrc`) file in an editor,
/// and make sure the line `SocksPort 9050` is uncommented.
/// You can then start Tor by running the Tor executable (`/usr/bin/tor`).
///
/// To check whether everything is working correctly, run `cargo run`.
/// If you want to use a special Tor address, you can pass it as first argument
/// or set the `TOR_PROXY` environment variable:
///
/// `cargo run -- 127.0.0.1:9050`
///
/// `TOR_PROXY=127.0.0.1:9050 cargo run`
///
/// [Tor installation guide]: https://www.torproject.org/docs/installguide.html.en
#[allow(unused)]
pub mod setup {}
