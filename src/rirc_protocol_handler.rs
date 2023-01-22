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

/// Public function handling protocol and sending each requests to the right function depending on the command
pub fn worker(connection: &mut MysqlConnection, request: Request, addr: String, thread_id: i32) -> Result<Response, IrcError> {
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
        USER => unimplemented(),
        SKIP => unimplemented(),
        WHOIS => whois(connection, request.content, thread_id),
        WHOWAS => whowas(connection, request.content, thread_id),
        _ => unimplemented(),
    }
}

fn is_banned(connection: &mut MysqlConnection, addr: String) -> bool {
    return match get_ban(connection, &true, addr.as_str()) {
        Ok(_) => true,
        Err(_) => false
    }
}


/// Returns a PONG to user
fn ping(content: String) -> Result<Response, IrcError> {
    Ok(Response::new("PONG :".to_string() + content.as_str()))
}

/// User logging in
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
                let res = Response::new(":localhost 001 ".to_string() + content.as_str() + " :Welcome!");
                Ok(res)
            }
        }
        Err(_) => {
            // Username has never logged in
            create_user(connection, content.as_str(), addr.as_str(), &true, &thread_id);
            let res = Response::new(":localhost 001 ".to_string() + content.as_str() + ":Welcome!");
            Ok(res)
        }
    }
}

/// User quitting server
fn quit(connection: &mut MysqlConnection, thread_id: i32) -> Result<Response, IrcError> {
    edit_user_from_thread_id(connection, &thread_id, &false).unwrap();

    // Send an empty response, we don't care about it
    Ok(Response::new("BYE BYE".to_string()))
}

fn unimplemented() -> Result<Response, IrcError> {
    Err(None)
}