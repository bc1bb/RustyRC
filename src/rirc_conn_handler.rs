//! RustyRC Connection Handler

use std::io::{BufRead, BufReader, Write};
use std::iter::Skip;
use std::net::{IpAddr, TcpStream};
use std::thread::sleep;
use std::time::Duration;
use diesel::MysqlConnection;
use log::{debug, trace};
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

    let ignored_requests = [CAP];

    loop {
        let reader = BufReader::new(stream.try_clone().unwrap());

        // For every line sent to server,
        // send request to worker()
        for line in reader.lines() {
            let line = line.unwrap();
            trace!("f{} [{}]: {}", addr, thread_id.to_string(), line.clone());

            let request = Request::new(line).unwrap();

            // Skip ignored commands
            if ! ignored_requests.contains(&request.command) {
                match worker(connection, request, addr.to_string(), thread_id) {
                    Ok(res) => {
                        sender(stream.try_clone().unwrap(), res.clone(), addr, thread_id);
                        if res.content == "BYE BYE" { return }
                    }
                    Err(err) => {
                        let res = Response::new(connection, err.to_u32(), thread_id, err.to_str().to_string());
                        sender(stream.try_clone().unwrap(), res, addr, thread_id);
                        if err == YoureBannedCreep { return }
                    }
                }
            }
        }
    }
}

fn sender(mut stream: TcpStream, response: Response, addr: IpAddr, thread_id: i32) {
    let line = ":".to_string() + response.server_name.as_str() + " " + response.numeric_reply.to_string().as_str() + " " + response.destination.as_str() + " " + response.content.as_str() + "\n";

    trace!("t{} [{}]: {}", addr, thread_id.to_string(), line.clone());

    stream.write(
            line
            .as_ref()).unwrap();
}