//! # RustyRC Protocol Handler
//!
//! File containing functions that will interpret commands as sent by clients, each command has it's own function
//!
//! Currently supports most critical commands, WIP for more...

use std::net::TcpStream;
use diesel::MysqlConnection;
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
        JOIN => join(connection, thread_id, request.clone().content, stream),
        MOTD => unimplemented(), // TODO
        NAMES => names(connection, thread_id, request.clone().content),
        NICK => nick(connection, request.content, addr, thread_id),
        PART => part(connection, thread_id, request.clone().content),
        PING => ping(request.content),
        PONG => unimplemented(), // Don't reply to pongs otherwise we will just massively ping pong all day
        PRIVMSG => privmsg(connection, thread_id, request.content),
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
    let line = create_user_line(user.clone(), "JOIN :") + channel;

    // Sending
    add_message(connection, channel, line.as_str()).unwrap();

    // Add membership to the table, so child thread knows what to do
    create_membership(connection, user.nick.as_str(), channel);

    spawn(|| {
        let connection = &mut establish_connection();

        // Thread will finally start waiting for messages
        wait_for_message(connection, stream);
    });

    // Preparing to return channel's MOTD to user
    let motd = get_channel(connection, channel).unwrap().motd;
    let line = "332 :".to_string() + motd.as_str();

    let res = line + "\n" + names(connection, thread_id, channel.to_string()).unwrap().content.as_str();

    Ok(Response::new(res))
}

/// Replying to NAMES commands,
///
/// Without argument (empty `content`) it will print all channels and logged users,
///
/// With an argument it will print users in said channel.
fn names(connection: &mut MysqlConnection, thread_id: i32, content: String) -> Result<Response,IrcError> {
    // Expecting input as (RFC1459):
    // NAMES [<channel>{,<channel>}]

    // RPL_NAMREPLY: 353
    // RPL_ENDOFNAMES: 366

    let user = get_user_from_thread_id(connection, &thread_id).unwrap();
    let mut res_string = "".to_string();

    // expecting answer for all channels
    if content.is_empty() {
        for channel in get_all_channels(connection).unwrap() {
            // 353 "<channel> :[[@|+]<nick> [[@|+]<nick> [...]]]"
            res_string = ":localhost 353 ".to_string() + user.nick.as_str() + " = " + channel.name.as_str() + " :";
            for membership in get_all_channel_memberships(connection, channel.id).unwrap() {
                res_string = res_string + get_user_from_id(connection, &membership.id_user).unwrap().nick.as_str() + " ";
            }

            res_string = res_string + "\n:localhost 366 " + user.nick.as_str() + " " + channel.name.as_str() + " :End of /NAMES list.";
        }
    }

    // expecting answer for specific channel
    if content.contains(",") {
        return Err(TooManyTargets);
    }

    let channel = content.as_str();

    // target channel doesnt exist
    if get_channel(connection, content.as_str()).is_err() {
        return Err(NoSuchChannel);
    }
    let channel_id = get_channel(connection, content.as_str()).unwrap().id;

    // 353 "<channel> :[[@|+]<nick> [[@|+]<nick> [...]]]"
    res_string = ":localhost 353 ".to_string() + user.nick.as_str() + " = " + channel + " :";
    for membership in get_all_channel_memberships(connection, channel_id).unwrap() {
        res_string = res_string + get_user_from_id(connection, &membership.id_user).unwrap().nick.as_str() + " ";
    }

    res_string = res_string + "\n:localhost 366 " + user.nick.as_str() + " " + channel + " :End of /NAMES list.";

    Ok(Response::new(res_string))
}

/// User logging in
fn nick(connection: &mut MysqlConnection, content: String, addr: String, thread_id: i32) -> Result<Response, IrcError> {
    let nick = first_word(content.as_str());

    check_nick(connection, nick)?;

    let db_user = get_user_from_nick(connection, nick);

    // if user already has a nickname
    match get_user_from_thread_id(connection, &thread_id) {
        Ok(user) => { set_connected(connection, user, &false) }
        Err(_) => {}
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

/// Handling user leaving a channel
fn part(connection: &mut MysqlConnection, thread_id: i32, content: String) -> Result<Response, IrcError> {
    let channel_str = first_word(content.as_str());

    let user = get_user_from_thread_id(connection, &thread_id).unwrap();

    if channel_str.contains(",") {
        return Err(TooManyChannels);
    }

    if get_channel(connection, channel_str).is_err() {
        return Err(NoSuchChannel);
    }
    let channel = get_channel(connection, channel_str).unwrap();

    let mut user_in_channel = false;

    for membership in get_all_user_memberships(connection, user.id).unwrap() {
        if membership.id_channel == channel.id {
            user_in_channel = true;
            break
        }
    }

    if ! user_in_channel {
        return Err(NotOnChannel);
    }

    let line = create_user_line(user, "PART") + channel_str;
    add_message(connection, channel.name.as_str(), line.as_str()).unwrap();

    Ok(Response::no_response())
}

/// Returns a PONG to client
fn ping(content: String) -> Result<Response, IrcError> {
    Ok(Response::new("PONG :".to_string() + content.as_str()))
}

/// Handling user sending message to channel
fn privmsg(connection: &mut MysqlConnection, thread_id: i32, content: String) -> Result<Response,IrcError> {
    // Expecting request in this form (RFC 1459):
    // PRIVMSG <receiver>{,<receiver>} <text to be sent>
    let mut content_vec: Vec<&str> = content.split_whitespace().collect();

    let mut receiver = content_vec[0];
    let receiver_with_hashtag = "#".to_string() + receiver.clone();

    // Testing channel as both #`receiver` and `receiver`
    // Because some irc client add #, some don't :DDDDDD
    if get_channel(connection, receiver).is_err() {
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

    let mut message = create_user_line(sender.clone(), "PRIVMSG ") + receiver + " :";

    for word in &mut content_vec[1..] {
        for char in word.chars() {
            message = message + char.to_string().as_str();
        }
        message = message + " ";
    }

    add_message(connection, receiver, &*message)?;

    Ok(Response::no_response())
}

/// User quitting server,
///
/// It will broadcast to all channels that user is leaving them.
fn quit(connection: &mut MysqlConnection, thread_id: i32) -> Result<Response, IrcError> {
    let user = get_user_from_thread_id(connection, &thread_id).unwrap();

    // [channel] gets replaced by whatever the channel name is inside the function `broadcast_as_user`
    let line = create_user_line(user.clone(), " PART [channel]");

    broadcast_as_user(connection, user.nick.as_str(), line.to_string()).unwrap();

    set_connected(connection, user.clone(), &false);

    delete_user_membership(connection, user);

    Ok(Response::new("BYE BYE".to_string()))
}

/// User logging in (part2).
///
/// Only really used to define real_name, other parameters are ignored.
///
/// Will drop an error if first argument (supposedly username) is not a known nickname.
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

    // if user doesnt exist or is not logged in
    let user = match get_user_from_nick(connection, nick) {
        Ok(user) => { if user.is_connected { user } else { return Err(UnknownError) }}
        Err(_) => { return  Err(UnknownError) }
    };

    set_real_name(connection, user, real_name.as_str());

    Ok(Response::new(":localhost 001 ".to_string() + nick + " :Real name stored..."))
}

/// Replying to WHOIS commands, will reply only if user is logged in
fn whois(connection: &mut MysqlConnection, content: String, w_thread_id: i32) -> Result<Response, IrcError> {
    let mut res = Response::new(":localhost ".to_string());

    let sender = get_user_from_thread_id(connection, &w_thread_id).unwrap().nick;

    match get_user_from_nick(connection, content.as_str()) {
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

/// Replying to WHOWAS commands, will reply no matter if user is logged in or not
fn whowas(connection: &mut MysqlConnection, content: String, w_thread_id: i32) -> Result<Response, IrcError> {
    let mut res = Response::new(":localhost ".to_string());

    let sender = get_user_from_thread_id(connection, &w_thread_id).unwrap().nick;

    match get_user_from_nick(connection, content.as_str()) {
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

// # Utility Functions #

/// Function used when clients call for unsupported commands
fn unimplemented() -> Result<Response, IrcError> {
    Ok(Response::no_response())
}

/// Function used to create a user line when user is leaving/joining channel/server or sending a message, in the form:
///
/// `:<nickname>!<nickname>@<last_ip> <content>`
fn create_user_line(user: User, content: &str) -> String {
    let nick = user.nick;
    let last_ip = user.last_ip;

    return ":".to_string() + nick.as_str() + "!" + nick.as_str() + "@" + last_ip.as_str() + " " + content
}

/// Checking if a nickname is valid,
/// - Less than 11 chars,
/// - Not banned,
/// - Does not contain special characters (even `_` are banned).
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

/// Checking if user is banned, returns a `bool`.
fn is_banned(connection: &mut MysqlConnection, addr: &str) -> bool {
    return match get_ban(connection, &true, addr) {
        Ok(_) => true,
        Err(_) => false
    }
}