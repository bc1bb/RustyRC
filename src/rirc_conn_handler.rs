//! RustyRC Connection Handler

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;
use log::debug;
use crate::rirc_lib::*;
use crate::rirc_lib::Commands::*;
use crate::rirc_lib::IrcError::*;

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
    let addr = stream.peer_addr().unwrap().ip();
    loop {
        let reader = BufReader::new(stream.try_clone().unwrap());

        // For every line sent to server,
        // send request to worker()
        for line in reader.lines() {
            let line = line.unwrap_or_else(|e| { panic!("{}", e) });

            // Ignore invalid request, they are most likely unimplemented stuff for now
            // TODO: implement more
            let request = Request::new(line).unwrap();

            worker(request, addr.to_string());
        }
    }
}

fn worker(request: Request, addr: String) {
    match request.command {
        CAP => return,
        NICK => nick(request.content, addr).unwrap(),
        PRIVMSG => {}
        JOIN => {}
        MOTD => {}
        PING => {}
        PONG => {}
        QUIT => {}
        SKIP => return,
        _ => return,
    };
}

fn nick(content: String, addr: String) -> Result<(), IrcError> {
    let connection = &mut establish_connection();
    let db_user = get_user(connection, content.as_str());


    return match db_user {
        Ok(_) => {
            // A user with same name is already logged in
            if db_user.unwrap().is_connected { Err(NicknameInUse) }
            else { Ok(()) }
        }
        Err(_) => {
            create_user(connection, content.as_str(), addr.as_str(), &true);
            Ok(())
        }
    }
}