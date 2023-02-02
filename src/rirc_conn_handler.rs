//! RustyRC Connection Handler

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use diesel::MysqlConnection;
use log::trace;
use crate::rirc_lib::*;
use crate::rirc_lib::Commands::*;
use crate::rirc_lib::IrcError::*;
use crate::rirc_protocol_handler::*;

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
pub fn handler(connection: &mut MysqlConnection, mut stream: TcpStream, thread_id: i32) {
    let addr = stream.peer_addr().unwrap().ip();

    loop {
        let reader = BufReader::new(stream.try_clone().unwrap());

        // For every line sent to server,
        // send request to worker()
        for line in reader.lines() {
            let line = line.unwrap();
            trace!("f{} [{}]: {}", addr, thread_id.to_string(), line.clone());

            let request = Request::new(line).unwrap();

            // Skip CAP commands
            if request.clone().command != CAP {
                match worker(connection, request, addr.to_string(), thread_id) {
                    Ok(res) => {
                        // if request is QUIT
                        if res.content == "BYE BYE" { return }
                        else { sender(stream.try_clone().unwrap(), thread_id, res); }
                    }
                    Err(err) => {
                        let res = Response::new(err.to_u32().to_string() + " " + err.to_str());
                        sender(stream.try_clone().unwrap(), thread_id, res);
                        if err == YoureBannedCreep { return }
                    }
                }
            }
        }
    }
}

/// Simple function `writing` to `TcpStream`,
///
/// It is making sure that we send our responses with a \n at the end.
fn sender(mut stream: TcpStream, thread_id: i32, response: Response) {
    let line = response.content;

    trace!("t{} [{}]: {}", stream.peer_addr().unwrap(), thread_id.to_string(), line);
    stream.write((line + "\n").as_ref()).unwrap();
}