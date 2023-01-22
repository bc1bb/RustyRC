//! RustyRC Connection Handler

use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, TcpStream};
use std::thread::sleep;
use std::time::Duration;
use diesel::MysqlConnection;
use log::{debug, trace};
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
pub fn handler(connection: &mut MysqlConnection, mut stream: TcpStream, thread_id: i32) {
    let addr = stream.peer_addr().unwrap().ip();

    loop {
        let reader = BufReader::new(stream.try_clone().unwrap());

        // For every line sent to server,
        // send request to worker()
        for line in reader.lines() {
            let line = line.unwrap_or_else(|e| { panic!("{}", e) });
            trace!("{} [{}]: {}", addr, thread_id.to_string(), line.clone());

            let request = Request::new(line).unwrap();

            // Skip CAP commands
            if request.clone().command != CAP {
                match worker(connection, request, addr.to_string(), thread_id) {
                    Ok(res) => {
                        if res.content == "BYE BYE" { return }
                        else { stream.write((res.content + "\n").as_ref()).unwrap(); }
                    }
                    Err(err) => {
                        stream.write((err.to_u32().to_string() + " " + err.to_str() + "\n").as_ref()).unwrap();
                        if err == YoureBannedCreep { return }
                    }
                }
            }
        }
    }
}

fn is_banned(connection: &mut MysqlConnection, addr: String) -> bool {
    return match get_ban(connection, &true, addr.as_str()) {
        Ok(_) => true,
        Err(_) => false
    }
}

fn worker(connection: &mut MysqlConnection, request: Request, addr: String, thread_id: i32) -> Result<Response, IrcError> {
    if is_banned(connection, addr.clone()) {
        return Err(YoureBannedCreep);
    }

    return match request.command {
        NICK => nick(connection, request.content, addr, thread_id),
        PRIVMSG => unimplemented(),
        JOIN => unimplemented(),
        MOTD => unimplemented(),
        PING => ping(request.content),
        PONG => unimplemented(),
        QUIT => quit(connection, thread_id),
        USER => user(connection, request.content),
        SKIP => unimplemented(),
        _ => unimplemented(),
    }
}

fn ping(content: String) -> Result<Response, IrcError> {
    Ok(Response::new("PONG :".to_string() + content.as_str()))
}

fn user(connection: &mut MysqlConnection, content: String) -> Result<Response, IrcError> {
    Ok(Response::new(":localhost 001 guillaume :Welcome!".to_string()))
}

fn nick(connection: &mut MysqlConnection, content: String, addr: String, thread_id: i32) -> Result<Response, IrcError> {
    let db_user = get_user(connection, content.as_str());


    return match db_user {
        Ok(_) => {
            if db_user.unwrap().is_connected {
                // A user with same name is already logged in
                Err(NicknameInUse)
            } else {
                // A user with same name has already logged in but logged off since then
                edit_user(connection, content.as_str(), addr.as_str(), &true, &thread_id).unwrap();
                Ok(Response::new(":localhost 001 guillaume :Welcome!".to_string()))
            }
        }
        Err(_) => {
            // Username has never logged in
            create_user(connection, content.as_str(), addr.as_str(), &true, &thread_id);
            Ok(Response::new(":localhost 001 guillaume :Welcome!".to_string()))
        }
    }
}

fn quit(connection: &mut MysqlConnection, thread_id: i32) -> Result<Response, IrcError> {
    edit_user_from_thread_id(connection, &thread_id, &false).unwrap();

    // Send an empty response, we don't care about it
    Ok(Response::new("BYE BYE".to_string()))
}

fn unimplemented() -> Result<Response, IrcError> {
    Err(None)
}