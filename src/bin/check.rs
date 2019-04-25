extern crate tor_stream;

use tor_stream::*;

use std::env::{args, var};
use std::io::{ErrorKind, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs};
use std::process::exit;

fn main() {
    let address = if let Some(address) = args().nth(1).or_else(|| var("TOR_PROXY").ok()) {
        address
            .to_socket_addrs()
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse socket address: {}", e);
                exit(1);
            })
            .next()
            .unwrap()
    } else {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050))
    };

    println!("Tor address {}", address);

    check_clear_web(address.clone());
    check_hidden_service(address);

    exit(0);
}

fn check_clear_web(address: SocketAddr) {
    let mut stream =
        TorStream::connect_with_address(address, "www.example.com:80").unwrap_or_else(|e| {
            if e.kind() == ErrorKind::ConnectionRefused {
                eprintln!("Connection refused, is Tor running?");
            } else {
                eprintln!("Failed to connect: {}", e);
            };
            exit(1);
        });

    stream
        .write_all(b"GET / HTTP/1.1\r\nConnection: Close\r\nHost: www.example.com\r\n\r\n")
        .expect("Failed to send request");

    let mut buf = String::with_capacity(1633);
    stream
        .read_to_string(&mut buf)
        .expect("Failed to read response");

    if buf.starts_with("HTTP/1.1 200 OK") {
        println!("Clear web check successful");
    } else {
        eprintln!(
            "Clear web check failed\nInvalid response; dump ({} bytes):\n--------\n{}\n--------",
            buf.len(),
            buf
        );
    }
}

fn check_hidden_service(address: SocketAddr) {
    let mut stream = TorStream::connect_with_address(address, ("wlupld3ptjvsgwqw.onion", 80))
        .unwrap_or_else(|e| {
            if e.kind() == ErrorKind::ConnectionRefused {
                eprintln!("Connection refused, is Tor running?");
            } else {
                eprintln!("Failed to connect: {}", e);
            };
            exit(1);
        });

    stream
        .write_all(b"GET / HTTP/1.1\r\nConnection: Close\r\nHost: wlupld3ptjvsgwqw.onion\r\n\r\n")
        .expect("Failed to send request");

    let mut buf = String::with_capacity(390);
    stream
        .read_to_string(&mut buf)
        .expect("Failed to read response");

    if buf.starts_with("HTTP/1.1 200 OK") {
        println!("Hidden service check successful");
    } else {
        eprintln!("Hidden service check failed\nInvalid response; dump ({} bytes):\n--------\n{}\n--------", buf.len(), buf);
    }
}
