//! RustyRC Connection Handler

use std::net::TcpStream;
use diesel::MysqlConnection;
use log::trace;
use crate::rirc_lib::*;
use crate::rirc_lib::Commands::*;
use crate::rirc_lib::IrcError::*;
use crate::rirc_message_handler::wait_for_message;
use std::thread::spawn;

/// Public function handling protocol and sending each requests to the right function depending on the command
pub fn worker(connection: &mut MysqlConnection, request: Request, addr: String, thread_id: i32, stream: TcpStream) -> Result<Response, IrcError> {
    if is_banned(connection, addr.as_str()) {
        return Err(YoureBannedCreep);
    }

    return match request.command {
        NICK => nick(connection, request.content, addr, thread_id),
        PRIVMSG => privmsg(connection, thread_id, request.content),
        JOIN => join(connection, thread_id, request.clone().content, stream),
        MOTD => unimplemented(), // TODO
        PING => ping(request.content),
        PONG => unimplemented(), // Don't reply to pongs otherwise we will just massively ping pong all day
        QUIT => quit(connection, thread_id),
        USER => user(connection, request.content),
        WHOIS => whois(connection, request.content, thread_id),
        WHOWAS => whowas(connection, request.content, thread_id),

        // TODO: KICK, KILL, USERS, SERVLIST (?)
        _ => unimplemented(),
    }
}

/// Handling users joining channels
fn join(connection: &mut MysqlConnection, thread_id: i32, content: String, stream: TcpStream) -> Result<Response,IrcError> {
    // Expecting message such as
    // JOIN <channel>{,<channel>} [<key>{,<key>}]

    let binding = content.clone(); // ???????? RUST HELLO ???
    let channel = first_word(&binding);

    if channel.contains(",") {
        return Err(TooManyChannels)
    }

    if get_channel(connection, channel).is_err() {
        return Err(NoSuchChannel)
    }

    // Preparing to send a message such as ":WiZ JOIN #Twilight_zone" in the channel
    let user = get_user_from_thread_id(connection, &thread_id).unwrap();
    let line = create_user_line(user, "JOIN ") + channel;

    // Sending
    add_message(connection, channel, nick.as_str(), line.as_str()).unwrap();

    // Add membership to the table, so child thread knows what to do
    create_membership(connection, nick.as_str(), channel);

    spawn(|| {
        let connection = &mut establish_connection();

        // Thread will finally start waiting for messages
        wait_for_message(connection, stream);
    });

    // Preparing to return channel's MOTD to user
    let motd = get_channel(connection, channel).unwrap().motd;
    let line = "332 :".to_string() + motd.as_str();

    Ok(Response::new(line))
}

/// Handling user sending message to channel
fn privmsg(connection: &mut MysqlConnection, thread_id: i32, content: String) -> Result<Response,IrcError> {
    // Expecting request in this form (RFC 1459):
    // PRIVMSG <receiver>{,<receiver>} <text to be sent>
    let mut content_vec: Vec<&str> = content.split_whitespace().collect();

    let mut receiver = content_vec[0];
    let receiver_with_hashtag = "#".to_string() + receiver.clone();

    // Testing receiver as both user and channel, also testing channel as both #`receiver` and `receiver`
    // Because some irc client add #, some don't :DDDDDD
    if get_user(connection, receiver).is_err() && get_channel(connection, receiver).is_err() {
        if get_channel(connection, receiver_with_hashtag.as_str()).is_ok() {
            receiver = receiver_with_hashtag.as_str();
        } else {
            return Err(NoSuchChannel)
        }
    }

    let sender = get_user_from_thread_id(connection, &thread_id).unwrap();

    // We won't handle sending to multiple recipients
    if receiver.contains(",") {
        return Err(TooManyTargets)
    }

    let mut message = create_user_line(sender, "PRIVMSG " + receiver + " :");

    for word in &mut content_vec[1..] {
        for char in word.chars() {
            message = message + char.to_string().as_str();
        }
        message = message + " ";
    }

    add_message(connection, receiver, &*sender.nick, &*message)?;

    Ok(Response::new("".to_string()))
}

/// Checking if user is banned, returns a `bool`.
fn is_banned(connection: &mut MysqlConnection, addr: &str) -> bool {
    return match get_ban(connection, &true, addr) {
        Ok(_) => true,
        Err(_) => false
    }
}

/// Checking if a nickname is valid,
/// - Less than 11 chars,
/// - Not banned,
/// - Does not contain special characters.
fn check_nick(connection: &mut MysqlConnection, nick: &str) -> Result<(), IrcError> {
    // Is username banned ?
    if get_ban(connection, &false, nick).is_ok() {
        return Err(YoureBannedCreep);
    }

    // Is username longer than 11 characters ?
    if nick.len() > 11 {
        return Err(ErroneusNickname);
    }

    // Is username made of alphanumeric characters ?
    if ! nick.chars().all(char::is_alphanumeric) {
        return Err(ErroneusNickname);
    }

    return Ok(());
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

/// Replying to WHOWAS commands
fn whowas(connection: &mut MysqlConnection, content: String, w_thread_id: i32) -> Result<Response, IrcError> {
    let mut res = Response::new(":localhost ".to_string());

    let sender = get_user_from_thread_id(connection, &w_thread_id).unwrap().nick;

    match get_user(connection, content.as_str()) {
        Ok(user) => {
            // if user has ever existed
            res.content = res.content + "314 " + user.nick.as_str() + " " + user.nick.as_str() + " " + user.last_ip.as_str() + " " + user.real_name.as_str()
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

    check_nick(connection, nick)?;

    let db_user = get_user(connection, nick);

    // if user already has a nickname
    if get_user_from_thread_id(connection, &thread_id).is_ok() {
        set_connected_from_thread_id(connection, &thread_id, &false).unwrap();
    }

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
            let res = Response::new(":localhost 001 ".to_string() + nick + " :Welcome!");
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
    let mut real_name = "".to_string();
    if content_vec[3].starts_with(":") {
        // collect all parts into real_name
        for word in &mut content_vec[3..] {
            for char in word.chars() {
                real_name = real_name + char.to_string().as_str();
            }
            real_name = real_name + " ";
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
    let user = get_user_from_thread_id(connection, &thread_id).unwrap();

    set_connected_from_thread_id(connection, &thread_id, &false).unwrap();

    // [channel] gets replaced by whatever the channel name is inside the function `broadcast_as_user`
    let line = ":".to_string() + user.nick.as_str() + " PART [channel]";

    broadcast_as_user(connection, user.nick.as_str(), line.to_string()).unwrap();

    Ok(Response::new("BYE BYE".to_string()))
}

/// Function used when clients call for unsupported commands
///
/// Sending an empty response wil make sender() not send anything
fn unimplemented() -> Result<Response, IrcError> {
    Ok(Response::new("".to_string()))
}

/// Function used to create a user line when user is leaving/joining channel/server or sending a message
fn create_user_line(user: User, content: &str) -> String {
    let nick = user.nick;
    let last_ip = user.last_ip;

    return ":".to_string() + nick.as_str() + "!" + nick.as_str() + "@" + last_ip.as_str() + " " + content
}