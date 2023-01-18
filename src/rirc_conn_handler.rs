//! RustyRC Connection Handler

use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;
use diesel::MysqlConnection;
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
pub fn handler(connection: &mut MysqlConnection, mut stream: TcpStream) {
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

            let status = worker(connection, request, addr.to_string());

            if status.is_err() {
                stream.write(
                    (status.err().unwrap().to_u32().to_string() + "\n\n").as_ref()
                ).unwrap();
            }
        }
    }
}

fn worker(connection: &mut MysqlConnection, request: Request, addr: String) -> Result<(), IrcError> {
    return match request.command {
        CAP => Ok(()), // Skip CAP commands
        NICK => nick(connection, request.content, addr),
        PRIVMSG => Ok(()),
        JOIN => Ok(()),
        MOTD => Ok(()),
        PING => Ok(()),
        PONG => Ok(()),
        QUIT => Ok(()),
        SKIP => Ok(()),
        _ => Ok(()),
    }
}

fn nick(connection: &mut MysqlConnection, content: String, addr: String) -> Result<(), IrcError> {
    let db_user = get_user(connection, content.as_str());


    return match db_user {
        Ok(_) => {
            if db_user.unwrap().is_connected {
                // A user with same name is already logged in
                Err(NicknameInUse)
            } else {
                // A user with same name has already logged in but logged off since then
                edit_user(connection, content.as_str(), addr.as_str(), &true).unwrap();
                Ok(())
            }
        }
        Err(_) => {
            // Username has never logged in
            create_user(connection, content.as_str(), addr.as_str(), &true);
            Ok(())
        }
    }
}