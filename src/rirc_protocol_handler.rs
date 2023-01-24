//! RustyRC Connection Handler

use std::io::{BufRead, BufReader, Write};
use std::net::{IpAddr, TcpStream};
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
        USER => user(connection, request.content),
        SKIP => unimplemented(),
        WHOIS => whois(connection, request.content, thread_id),
        WHOWAS => whowas(connection, request.content, thread_id),
        _ => unimplemented(),
    }
}

/// Checking if user is banned, returns a `bool`.
fn is_banned(connection: &mut MysqlConnection, addr: String) -> bool {
    return match get_ban(connection, &true, addr.as_str()) {
        Ok(_) => true,
        Err(_) => false
    }
}

/// Replying to WHOIS commands
fn whois(connection: &mut MysqlConnection, content: String, w_thread_id: i32) -> Result<Response, IrcError> {
    let mut res = Response::new(":localhost ".to_string());

    let sender = get_user_from_thread_id(connection, &w_thread_id).unwrap().nick;

    match get_user(connection, content.as_str()) {
        Ok(user) => {
            if user.is_connected {
                // User is currently logged in
                res.content = res.content + "311 " + user.nick.as_str() + " " + user.nick.as_str() + " " + user.last_ip.as_str() + " " + user.real_name.as_str()
            } // User is not currently logged in
            else { res.content = res.content + "401 " + sender.as_str() + " " + content.as_str() + " :No such nick registered" }
        }
        // User has never logged in
        Err(_) => res.content = res.content + "401 " + sender.as_str() + " " + content.as_str() + " :No such nick registered",
    }

    res.content = res.content + "\n:localhost 318 " + sender.as_str() + " " + content.as_str() + " :End of /WHOIS";

    Ok(res)
}

fn whowas(connection: &mut MysqlConnection, content: String, w_thread_id: i32) -> Result<Response, IrcError> {
    let mut res = Response::new(":localhost ".to_string());

    let sender = get_user_from_thread_id(connection, &w_thread_id).unwrap().nick;

    match get_user(connection, content.as_str()) {
        Ok(user) => {
            if user.is_connected {
                // User is currently logged in
                res.content = res.content + user.nick.as_str() + "@" + user.last_ip.as_str()
            } // User is not currently logged in
            else { res.content = res.content + user.nick.as_str() + "@" + user.last_ip.as_str() }
        }
        // User has never logged in
        Err(_) => res.content = res.content + "406 " + sender.as_str() + " " + content.as_str() + " :No such nick registered",
    }

    res.content = res.content + "\n:localhost 369 " + sender.as_str() + " " + content.as_str() + " :End of /WHOWAS";

    Ok(res)
}

/// Returns a PONG to client
fn ping(content: String) -> Result<Response, IrcError> {
    Ok(Response::new("PONG :".to_string() + content.as_str()))
}

/// User logging in
fn nick(connection: &mut MysqlConnection, content: String, addr: String, thread_id: i32) -> Result<Response, IrcError> {
    let nick = first_word(content.as_str());

    let db_user = get_user(connection, nick);

    return match db_user {
        Ok(db_user) => {
            if db_user.is_connected {
                // A user with same name is already logged in
                Err(NicknameInUse)
            } else {
                // A user with same name has already logged in but logged off since then
                edit_user(connection, &get_current_epoch(), nick, addr.as_str(), &true, &thread_id).unwrap();
                let res = Response::new(":localhost 001 ".to_string() + nick + " :Welcome!");
                Ok(res)
            }
        }
        Err(_) => {
            // Username has never logged in
            create_user(connection, &get_current_epoch(), nick, nick, addr.as_str(), &true, &false, &thread_id);
            let res = Response::new(":localhost 001 ".to_string() + nick + ":Welcome!");
            Ok(res)
        }
    }
}

/// User logging in (part2)
///
/// Only really used to define real_name, other parameters are ignored.
fn user(connection: &mut MysqlConnection, content: String) -> Result<Response, IrcError> {
    let content_iter = content.split(" ");
    // Expected form: (from RFC1459)
    // <username> <hostname> <servername> <realname>
    let mut content_vec = content_iter.collect::<Vec<_>>();

    let nick = content_vec[0];

    if ! content_vec.len() <= 4 {
        return Err(NeedMoreParams)
    }

    // if <realname> starts with ":" (declaring a multi word string)
    let mut real_name: String;
    if content_vec[3].starts_with(":") {
        // collect all parts into real_name
        real_name = content_vec[3].to_string();
        for parts in &mut content_vec[4..] {
            real_name = real_name + " " + parts;
        }
    } else {
        real_name = content_vec[3].to_string();
    }

    // if user doesnt exist
    let user = get_user(connection, nick);
    if user.is_err() {
        return Err(UnknownError)
    }
    // if user exist but is not logged in
    if ! user.unwrap().is_connected {
        return Err(UnknownError)
    }

    set_real_name(connection, nick, real_name.as_str()).unwrap();

    Ok(Response::new(":localhost 001 ".to_string() + nick + " :Real name stored..."))
}

/// User quitting server
fn quit(connection: &mut MysqlConnection, thread_id: i32) -> Result<Response, IrcError> {
    set_connected_from_thread_id(connection, &thread_id, &false).unwrap();

    Ok(Response::new("BYE BYE".to_string()))
}

fn unimplemented() -> Result<Response, IrcError> {
    Err(None)
}