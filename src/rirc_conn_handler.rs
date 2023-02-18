//! # RustyRC Connection Handler
//!
//! File containing functions working on the connection itself.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use diesel::MysqlConnection;
use log::trace;
use crate::rirc_lib::*;

use crate::rirc_lib::IrcError::*;
use crate::rirc_protocol_handler::*;

/// Public function that handles `TcpStream`,
/// each lines sent to `handler` are sent to `rirc_protocol_handler::worker()`,
/// which will try to figure out how to answer to commands.
///
/// Example:
/// ```rust
/// let listener = TcpListener::bind(SocketAddr::new("127.0.0.1", 6667)).unwrap();
///
/// for stream in listener.incoming() {
///     handler(stream.unwrap())
/// }
/// ```
pub fn handler(connection: &mut MysqlConnection, stream: TcpStream, thread_id: i32) {
    let addr = stream.peer_addr().unwrap().ip();

    loop {
        let reader = BufReader::new(stream.try_clone().unwrap());

        // For every line sent to server,
        // send request to worker()
        for line in reader.lines() {
            let line = line.unwrap();
            trace!("{}: {}", addr, line.clone());

            let request = Request::new(line).unwrap();

            match worker(connection, request, addr.to_string(), thread_id, stream.try_clone().unwrap()) {
                Ok(res) => {
                    // if request is QUIT
                    if res.content == "BYE BYE" { return }

                    sender(stream.try_clone().unwrap(), res);
                }
                Err(error) => {
                    // if error means user is banned, close connection
                    if error == YoureBannedCreep { return }

                    let res = Response::from_error(error);

                    sender(stream.try_clone().unwrap(), res);
                }
            }
        }
    }
}

/// Simple function `write`ing to `TcpStream`,
///
/// - It is making sure that we send our responses with a \n at the end,
/// - Will not send anything if `response.content` is empty,
/// - Send a `trace!()` for every line sent.
pub fn sender(mut stream: TcpStream, response: Response) {
    let line = response.content;

    if line == "" {
        return
    }

    trace!("{}: {}", stream.peer_addr().unwrap(), line);
    stream.write((line + "\n").as_ref()).unwrap();
}