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
        PING => ping(connection, thread_id, request.content),
        QUIT => quit(connection, thread_id),
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

/// Replying to WHOIS commands
fn whois(connection: &mut MysqlConnection, content: String, w_thread_id: i32) -> Result<Response, IrcError> {
    let mut res: Response;

    let sender = get_user_from_thread_id(connection, &w_thread_id).unwrap().name;

    match get_user(connection, content.as_str()) {
        Ok(user) => {
            if user.is_connected {
                // User is currently logged in
                res = Response::new(connection, 311, w_thread_id, (user.name + "@" + user.last_ip.as_str()));
            } // User is not currently logged in
            else { res = Response::new(connection, 401, w_thread_id, ":No such nick logged in".to_string()) }
        }
        // User has never logged in
        Err(_) => res = Response::new(connection, 401, w_thread_id, ":No such nick registered".to_string()),
    }

    // Hack to send two responses at once
    res.content = res.content + "\n:localhost 318 " + sender.as_str() + " " + content.as_str() + " :End of /WHOIS";

    Ok(res)
}

/// Replying to WHOWAS commands
fn whowas(connection: &mut MysqlConnection, content: String, w_thread_id: i32) -> Result<Response, IrcError> {
    let mut res: Response;

    let sender = get_user_from_thread_id(connection, &w_thread_id).unwrap().name;

    match get_user(connection, content.as_str()) {
        Ok(user) => {
            if user.is_connected {
                // User is currently logged in
                res = Response::new(connection, 311, w_thread_id, (user.name + "@" + user.last_ip.as_str()));
            } // User is not currently logged in
            else { res = Response::new(connection, 314, w_thread_id, (user.name + "@" + user.last_ip.as_str())); }
        }
        // User has never logged in
        Err(_) => res = Response::new(connection, 401, w_thread_id, ":No such nick registered".to_string()),
    }

    // Hack to send two responses at once
    res.content = res.content + "\n:localhost 369 " + sender.as_str() + " " + content.as_str() + " :End of /WHOWAS";

    Ok(res)
}

/// Returns a PONG to user
fn ping(connection: &mut MysqlConnection, w_thread_id: i32, content: String) -> Result<Response, IrcError> {
    let res = Response::new(connection, 001, w_thread_id, ("PONG".to_string() + content.as_str()));

    Ok(res)
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
                let res = Response::new(connection, 001, thread_id, " :Welcome!".to_string());
                Ok(res)
            }
        }
        Err(_) => {
            // Username has never logged in
            create_user(connection, content.as_str(), addr.as_str(), &true, &thread_id);
            let res = Response::new(connection, 001, thread_id, " :Welcome!".to_string());
            Ok(res)
        }
    }
}

/// User quitting server
fn quit(connection: &mut MysqlConnection, thread_id: i32) -> Result<Response, IrcError> {
    edit_user_from_thread_id(connection, &thread_id, &false).unwrap();

    // Send an empty response, we don't care about it
    Ok(Response::new(connection, 420, thread_id, "BYE BYE".to_string()))
}

fn unimplemented() -> Result<Response, IrcError> {
    Err(None)
}