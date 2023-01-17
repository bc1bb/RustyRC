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
    // TODO: MAKE THIS LESS BASIC ???
    // Works with irssi
    loop {
        let reader = BufReader::new(stream.try_clone().unwrap());

        let mut i = 0;
        for line in reader.lines() {
            i += 1;
            let line = line.unwrap_or_else(|e| { panic!("{}", e) });
            println!("{} {}", i, line);

            // client says i'm connected
            // (CAP LS & NICK xxx & USER xxxx localhost : RealName)
            if i == 3 {
                stream.write(":localhost 001 guillaume :Welcome!\n".as_ref()).unwrap();
                println!("non");
            }

            // client pings
            if i == 5 {
                stream.write("PONG localhost\n".as_ref()).unwrap();
                println!("non");
            }
        }
    }
}