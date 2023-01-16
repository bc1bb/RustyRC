//! RustyRC Connection Handler

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;
use log::debug;

/// Public function that handles `TcpStream`,
///
/// Example:
/// ```rust
/// let listener = TcpListener::bind(SocketAddr::new("127.0.0.1", 6667)).unwrap();
///
/// for stream in listener.incoming() {
///     handler(stream.unwrap())
/// }
/// ```
pub fn handler(mut stream: TcpStream) {
    loop {
        debug!("PINGED !");
        stream.write("PING: 1234".as_ref()).unwrap();

        let buf_reader = BufReader::new(&mut stream);

        let received: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        for (i, line) in received.iter().enumerate() {
            if line == &"PONG: 1234".to_string() {
                debug!("PONGED !");
            } else {
                return;
            }
        }
    }
}