//! RustyIRC Lib
//!
//! Shared file containing different structs and public functions for other modules to work.
//!
//! Including objects used for database communication.

use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec::IntoIter;
use diesel::dsl::not;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use dotenvy::dotenv;
use crate::rirc_lib::Error::*;
use crate::rirc_lib::IrcError::*;
use crate::rirc_schema::*;


/// Holding responses sent by server in a struct
#[derive(Clone)]
pub struct Response {
    pub content: String,
}

impl Response {
    pub fn new(content: String) -> Response {
        Response { content }
    }
}

/// Holding commands that can be handled by our server
#[derive(PartialEq, Clone)]
#[allow(dead_code)]
pub enum Commands {
    // Supported commands
    CAP, NICK, PART, PRIVMSG, JOIN, MOTD, NAMES, PING, PONG, QUIT, USER, WHOIS, WHOWAS,

    SKIP,

    // Unsupported commands
    ADMIN, AWAY, CNOTE, CONNECT, DIE, ENCAP, ERROR, HELP, INFO, INVITE, ISON, KICK, KILL,
    KNOCK, LINKS, LIST, LUSERS, MODE, NOTICE, OPER, PASS, REHASH, RULES, SERVER,
    SERVICE, SERVLIST, SQUERY, SQUIT, SETNAME, SILENCE, STATS, SUMMON, TIME, TOPIC, TRACE,
    USERHOST, USERIP, USERS, VERSION, WALLOPS, WATCH, WHO,
}

impl Commands {
    /// Public function returning the request type as `Commands` from a `&str`,
    ///
    /// Example: `from_str("CAP").unwrap();` will return `CAP`,
    ///
    /// Can return `Error::InvalidRequest`
    pub fn from_str(content: &str) -> Result<Commands, Error> {
        use self::Commands::*;
        match content {
            "CAP" => Ok(CAP),
            "NICK" => Ok(NICK),
            "PART" => Ok(PART),
            "PRIVMSG" => Ok(PRIVMSG),
            "JOIN" => Ok(JOIN),
            "MOTD" => Ok(MOTD),
            "NAMES" => Ok(NAMES),
            "PING" => Ok(PING),
            "PONG" => Ok(PONG),
            "QUIT" => Ok(QUIT),
            "USER" => Ok(USER),
            "WHOIS" => Ok(WHOIS),
            "WHOWAS" => Ok(WHOWAS),
            _ => Ok(SKIP),
        }
    }
}

/// Holding requests sent by clients in a struct
#[derive(Clone)]
pub struct Request {
    pub command: Commands,
    pub content: String,
}

impl Request {
    pub fn new(request: String) -> Result<Request, Error> {
        // Splitting request
        let binding = request.clone();
        let request_split = binding.split(" ");

        // Send first part of request to RequestType::from_str()
        let command_str  = request_split.clone().nth(0).unwrap();
        let command = Commands::from_str(command_str)?;

        // Generate `content` from split, skip first part
        let mut content = request_split.clone().nth(1).unwrap().to_string();
        for i in request_split.skip(2) {
            content = content.to_owned() + " " + i
        }

        Ok(Request {
            command,
            content,
        })
    }
}

/// Enum holding general errors in the project
#[derive(Debug)]
pub enum Error {
    InvalidRequest,
    NoResultInDatabase,
}

/// Enum holding errors about the IRC Protocol,
///
// Errors are sent in a response containing only their number
// https://www.rfc-editor.org/rfc/rfc1459#section-6
#[derive(Debug,PartialEq)]
pub enum IrcError {
    None, // (=unimplemented)
    UnknownError, // 400: ERR_UNKNOWNERROR
    NoSuchNick, // 401: ERR_NOSUCHNICK
    NoSuchChannel, // 403: ERR_NOSUCHCHANNEL
    CannotSendToChan, // 404: ERR_CANNOTSENDTOCHAN
    TooManyChannels, // 405: ERR_TOOMANYCHANNELS
    TooManyTargets, // 407: ERR_TOOMANYTARGETS
    ErroneusNickname, // 432: ERR_ERRONEUSNICKNAME
    NicknameInUse, // 433: ERR_NICKNAMEINUSE
    NotOnChannel, // 442: ERR_NOTONCHANNEL
    NeedMoreParams, // 461: ERR_NEEDMOREPARAMS
    YoureBannedCreep, // 465: ERR_YOUREBANNEDCREEP
    YouWillBeBanned, // 466: ERR_YOUWILLBEBANNED
}

impl IrcError {
    /// Public function returning `u32` corresponding to error name,
    ///
    /// Example: `IrcError::NicknameInUse.to_u32()`
    pub fn to_u32(&self) -> u32 {
        use self::IrcError::*;

        match self {
            None => 0,
            UnknownError => 400,
            NoSuchNick => 401,
            NoSuchChannel => 403,
            CannotSendToChan => 404,
            TooManyChannels => 405,
            TooManyTargets => 407,
            ErroneusNickname => 432,
            NicknameInUse => 433,
            NotOnChannel => 442,
            NeedMoreParams => 461,
            YoureBannedCreep => 465,
            YouWillBeBanned => 466,
        }
    }

    /// Public function returning `&str` corresponding to error name,
    ///
    /// Example: `IrcError::NicknameInUse.to_string()`
    pub fn to_str(&self) -> &str {
        use self::IrcError::*;

        match self {
            None => "", // 0
            UnknownError => ":Unknown Error", // 400
            NoSuchNick => ":No Such Nick", // 401
            NoSuchChannel => ":No Such Channel", // 403
            CannotSendToChan => ":Cannot Send To Chan", // 404
            TooManyChannels => ":Too Many Channels", // 405
            TooManyTargets => ":Too Many Targets", // 407
            ErroneusNickname => ":Erroneus Nickname", // 432
            NicknameInUse => ":Nickname In Use", // 433
            NotOnChannel => ":Not On Channel", // 442
            NeedMoreParams => ":Need More Params", // 461
            YoureBannedCreep => ":You're Banned, Creep", // 465
            YouWillBeBanned => ":You Will Be Banned", // 466
        }
    }
}

/// Public function that handles connecting to MySQL with Diesel using `DATABASE_URL`,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// let ip: Setting = get_setting(connection, "ip");
/// ```
pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

/// Queryable public struct linked to database using Diesel.
#[derive(Queryable,Clone)]
pub struct User {
    pub id: i32,
    pub last_login: i64,
    pub nick: String,
    pub real_name: String,
    pub last_ip: String,
    pub is_connected: bool,
    pub op: bool,
    pub thread_id: i32,
}

/// Insertable public struct linked to database using Diesel.
#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub last_login: &'a i64,
    pub nick: &'a str,
    pub real_name: &'a str,
    pub last_ip: &'a str,
    pub is_connected: &'a bool,
    pub op: &'a bool,
    pub thread_id: &'a i32,
}

/// Public function that will return a `User` when given its `nick`,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_user(connection, "johndoe");
/// ```
pub fn get_user<'a>(connection: &mut MysqlConnection,  w_nick: &str) -> Result<User, Error> {
    use crate::rirc_schema::users::dsl::*;

    let mut user = users
        .limit(1)
        .filter(nick.eq(w_nick))
        .load::<User>(connection)
        .expect("Error loading users")
        .into_iter();

    if user.len() > 0 {
        Ok(user.nth(0).unwrap())
    } else {
        Err(NoResultInDatabase)
    }
}

/// Public function that will return a `User` when given its `thread_id`,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_user_from_thread_id(connection, &24);
/// ```
pub fn get_user_from_thread_id<'a>(connection: &mut MysqlConnection,  w_thread_id: &i32) -> Result<User, Error> {
    use crate::rirc_schema::users::dsl::*;

    let mut user = users
        .limit(1)
        .filter(thread_id.eq(w_thread_id))
        .load::<User>(connection)
        .expect("Error loading users")
        .into_iter();

    if user.len() > 0 {
        Ok(user.nth(0).unwrap())
    } else {
        Err(NoResultInDatabase)
    }
}

/// Public function that will return a `User` when given its `id`,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_user_from_thread_id(connection, &24);
/// ```
pub fn get_user_from_id<'a>(connection: &mut MysqlConnection,  w_id: &i32) -> Result<User, Error> {
    use crate::rirc_schema::users::dsl::*;

    let mut user = users
        .limit(1)
        .filter(id.eq(w_id))
        .load::<User>(connection)
        .expect("Error loading users")
        .into_iter();

    if user.len() > 0 {
        Ok(user.nth(0).unwrap())
    } else {
        Err(NoResultInDatabase)
    }
}

/// Public function that handles creating users,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// create_user(connection, "johndoe", "1.2.3.4", &true);
/// ```
pub fn create_user(connection: &mut MysqlConnection,
                   w_last_login: &i64, w_nick: &str, w_real_name: &str,
                   w_last_ip: &str, w_is_connected: &bool, w_op: &bool,
                   w_thread_id: &i32) {
    use crate::rirc_schema::users::dsl::*;
    use crate::rirc_schema::users;

    let new_user = NewUser {
        last_login: w_last_login,
        nick: w_nick,
        real_name: w_real_name,
        last_ip: w_last_ip,
        is_connected: w_is_connected,
        op: w_op,
        thread_id: w_thread_id
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(connection)
        .expect("Error saving new user");
}

/// Public function that handles editing certain parts of an existing user from its username,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// edit_user(connection, &1674587646,"johndoe", "1.1.1.1", &true, &4);
/// ```
pub fn edit_user(connection: &mut MysqlConnection,
                 w_last_login: &i64, w_nick: &str, w_last_ip: &str,
                 w_is_connected: &bool, w_thread_id: &i32) -> Result<(), Error> {
    use crate::rirc_schema::users::dsl::*;
    use crate::rirc_schema::users;

    if get_user(connection, w_nick).is_err() {
        return Err(NoResultInDatabase);
    }

    diesel::update(users::table)
        .filter(nick.eq(w_nick))
        .set(last_login.eq(w_last_login))
        .execute(connection)
        .expect("Error editing user");

    diesel::update(users::table)
        .filter(nick.eq(w_nick))
        .set(last_ip.eq(w_last_ip))
        .execute(connection)
        .expect("Error editing user");

    diesel::update(users::table)
        .filter(nick.eq(w_nick))
        .set(is_connected.eq(w_is_connected))
        .execute(connection)
        .expect("Error editing user");

    diesel::update(users::table)
        .filter(nick.eq(w_nick))
        .set(thread_id.eq(w_thread_id))
        .execute(connection)
        .expect("Error editing user");

    Ok(())
}

/// Public function that set `is_connected` to `w_is_connected` from `thread_id`,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// set_connected_from_thread_id(connection, &23, &false);
/// ```
pub fn set_connected_from_thread_id(connection: &mut MysqlConnection,
                                    w_thread_id: &i32, w_is_connected: &bool) -> Result<(), Error> {
    use crate::rirc_schema::users::dsl::*;
    use crate::rirc_schema::users;

    if get_user_from_thread_id(connection, w_thread_id).is_err() {
        return Err(NoResultInDatabase);
    }

    diesel::update(users::table)
        .filter(thread_id.eq(w_thread_id))
        .set(is_connected.eq(w_is_connected))
        .execute(connection)
        .expect("Error editing user");

    // if we want to declare our user as logged off
    if ! w_is_connected {
        diesel::update(users::table)
            .filter(thread_id.eq(w_thread_id))
            .set(thread_id.eq(-1))
            .execute(connection)
            .expect("Error editing user");
    }

    Ok(())
}

/// Public function that sets `real_name` to `w_real_name` from `nick`,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// set_real_name(connection, "johndoe", "John Doe");
/// ```
pub fn set_real_name(connection: &mut MysqlConnection,
                     w_nick: &str, w_real_name: &str) -> Result<(), Error> {
    use crate::rirc_schema::users::dsl::*;
    use crate::rirc_schema::users;

    if get_user(connection, w_nick).is_err() {
        return Err(NoResultInDatabase);
    }

    diesel::update(users::table)
        .filter(nick.eq(w_nick))
        .set(real_name.eq(w_real_name))
        .execute(connection)
        .expect("Error editing user");

    Ok(())
}

/// Public function that cleans database, it will set all users to logged off and set all threads id to -1
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// clean_database(connection);
/// ```
pub fn clean_database(connection: &mut MysqlConnection) {
    use crate::rirc_schema::users::dsl::*;
    use crate::rirc_schema::users;

    use crate::rirc_schema::memberships::dsl::*;
    use crate::rirc_schema::memberships;

    diesel::update(users::table)
        .set(is_connected.eq(false))
        .execute(connection)
        .expect("Error editing user");

    diesel::update(users::table)
        .set(thread_id.eq(-1))
        .execute(connection)
        .expect("Error editing user");

    diesel::delete(memberships::table)
        .execute(connection)
        .expect("Error removing memberships");
}

/// Function used when manipulating timestamps (for channels and users),
///
/// Returns the current unix timestamp as i64, for easier calls to function asking i64 for timestamps.
pub fn get_current_epoch() -> i64 {
    i64::try_from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()).unwrap()
}

/// Queryable private struct linked to database using Diesel.
#[derive(Queryable)]
pub struct Ban {
    pub id: i32,
    pub is_ip: bool, // if ban is applied on ip, this will be set to 1
    pub content: String,
}

/// Insertable private struct linked to database using Diesel.
#[derive(Insertable)]
#[diesel(table_name = bans)]
struct NewBan<'a> {
    pub is_ip: &'a bool,
    pub content: &'a str,
}

/// Public function that will return a `Ban` when given, `is_ip` and `content`,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_ban(connection, &true, "1.1.1.1");
/// get_ban(connection, &false, "johndoe");
/// ```
pub fn get_ban<'a>(connection: &mut MysqlConnection, w_is_ip: &bool, w_name: &str) -> Result<Ban,Error> {
    use crate::rirc_schema::bans::dsl::*;

    let mut ban = bans
        .limit(1)
        .filter(is_ip.eq(w_is_ip))
        .filter(content.eq(w_name))
        .load::<Ban>(connection)
        .expect("Error loading bans")
        .into_iter();

    if ban.len() > 0 {
        Ok(ban.nth(0).unwrap())
    } else {
        Err(NoResultInDatabase)
    }
}

/// Public function that handles creating bans,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// create_ban(connection, &true, "1.2.3.4"); // does IP ban
/// create_ban(connection, &false, "johndoe"); // does name-based ban
/// ```
pub fn create_ban(connection: &mut MysqlConnection, is_ip: &bool, content: &str) {
    use crate::rirc_schema::bans;

    let new_ban = NewBan { is_ip, content };

    diesel::insert_into(bans::table)
        .values(&new_ban)
        .execute(connection)
        .expect("Error saving new ban");
}

/// Queryable public struct linked to database using Diesel.
#[derive(Queryable)]
pub struct Channel {
    pub id: i32,
    pub name: String,
    pub creation_time: i32,
    pub creator: String,
    pub motd: String,
    pub content: String,
}


/// Insertable private struct linked to database using Diesel.
#[derive(Insertable)]
#[diesel(table_name = channels)]
pub struct NewChannel<'a> {
    pub name: &'a str,
    pub creation_time: &'a i32,
    pub creator: &'a str,
    pub motd: &'a str,
    pub content: &'a str,
}

/// Public function that will return a `Channel` when given it's `name`
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_channel(connection, "name");
/// ```
pub fn get_channel<'a>(connection: &mut MysqlConnection, w_name: &str) -> Result<Channel, Error> {
    use crate::rirc_schema::channels::dsl::*;

    let mut channel = channels
        .limit(1)
        .filter(name.eq(w_name))
        .load::<Channel>(connection)
        .expect("Error loading channels")
        .into_iter();

    if channel.len() == 1 {
        Ok(channel.nth(0).unwrap())
    } else {
        Err(NoResultInDatabase)
    }
}

/// Public function that will return a `Channel` when given it's `id`
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_channel_from_id(connection, &2);
/// ```
pub fn get_channel_from_id<'a>(connection: &mut MysqlConnection, w_id: &i32) -> Result<Channel, Error> {
    use crate::rirc_schema::channels::dsl::*;

    let mut channel = channels
        .limit(1)
        .filter(id.eq(w_id))
        .load::<Channel>(connection)
        .expect("Error loading channels")
        .into_iter();

    if channel.len() == 1 {
        Ok(channel.nth(0).unwrap())
    } else {
        Err(NoResultInDatabase)
    }
}

// Public function that will return all `Channel`s,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_all_channels(connection);
/// ```
pub fn get_all_channels<'a>(connection: &mut MysqlConnection) -> Result<Vec<Channel>, Error> {
    use crate::rirc_schema::channels::dsl::*;

    let channel = channels
        .load::<Channel>(connection)
        .expect("Error loading users");

    if channel.len() > 0 {
        Ok(channel)
    } else {
        Err(NoResultInDatabase)
    }
}

/// Public function that handles creating channels,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// create_channel(connection, "world", 1673616716, "system", "Welcome to our cool channel #world", "system: Hello, World");
/// ```
pub fn create_channel(connection: &mut MysqlConnection, name: &str, creation_time: &i32, creator: &str, motd: &str, content: &str) {
    use crate::rirc_schema::channels;

    let new_channel = NewChannel { name, creation_time, creator, content, motd };

    diesel::insert_into(channels::table)
        .values(&new_channel)
        .execute(connection)
        .expect("Error saving new channel");
}

/// Function used to add message when user sends PRIVMSG command
pub fn add_message(connection: &mut MysqlConnection, channel: &str, w_content: &str) -> Result<(), IrcError> {
    use crate::rirc_schema::channels::dsl::*;
    use crate::rirc_schema::channels;

    // Channel doesnt exist
    if get_channel(connection, channel).is_err() {
        return Err(NoSuchChannel);
    }

    let line = w_content;
        //(nick.to_string() + " " + w_content);

    diesel::update(channels::table)
        .filter(name.eq(channel))
        .set(content.eq(line))
        .execute(connection)
        .expect("Error editing channel");

    Ok(())
}

/// Function used to send in every channel a user is in
pub fn broadcast_as_user(connection: &mut MysqlConnection, nick: &str, w_content: String) -> Result<(), IrcError> {
    let user = get_user(connection, nick).unwrap();
    let memberships = get_all_user_memberships(connection, user.id).unwrap();

    for membership in memberships {
        let channel = get_channel_from_id(connection, &membership.id_channel).unwrap().name;

        // cant turn the line into a fucking variable because this language hates me
        add_message(connection, channel.as_str(), w_content.clone().replace("[channel]", channel.as_str()).as_str()).unwrap();
    }

    Ok(())
}

/// Queryable public struct linked to database using Diesel.
#[derive(Queryable)]
pub struct Setting {
    pub id: i32,
    pub key: String,
    pub content: String,
}

/// Public function that will return a `Setting` when given it's `key`
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_setting(connection, "name");
/// ```
pub fn get_setting<'a>(connection: &mut MysqlConnection, w_key: &str) -> Result<Setting, Error> {
    use crate::rirc_schema::settings::dsl::*;

    let mut setting = settings
        .limit(1)
        .filter(key.eq(w_key))
        .load::<Setting>(connection)
        .expect("Error loading settings")
        .into_iter();

    if setting.len() > 0 {
        Ok(setting.nth(0).unwrap())
    } else {
        Err(NoResultInDatabase)
    }
}

/// Queryable public struct linked to database using Diesel.
#[derive(Queryable)]
pub struct Membership {
    pub id: i32,
    pub id_user: i32,
    pub id_channel: i32,
}


/// Insertable private struct linked to database using Diesel.
#[derive(Insertable)]
#[diesel(table_name = memberships)]
pub struct NewMembership<'a> {
    pub id_user: &'a i32,
    pub id_channel: &'a i32,
}

/// Public function used to return the latest channel membership created,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_last_membership(connection);
/// ```
pub fn get_last_membership<'a>(connection: &mut MysqlConnection) -> Result<Membership, Error> {
    use crate::rirc_schema::memberships::dsl::*;

    let mut membership = memberships
        .limit(1)
        .order(id.desc())
        .load::<Membership>(connection)
        .expect("Error loading memberships")
        .into_iter();

    if membership.len() > 0 {
        Ok(membership.nth(0).unwrap())
    } else {
        Err(NoResultInDatabase)
    }
}

/// Public function used to return all memberships linked to a certain user_id,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_all_user_memberships(connection, 12);
/// ```
pub fn get_all_user_memberships(connection: &mut MysqlConnection, w_id: i32) -> Result<Vec<Membership>, Error> {
    use crate::rirc_schema::memberships::dsl::*;

    let membership = memberships
        .filter(id_user.eq(w_id))
        .load::<Membership>(connection)
        .expect("Error loading memberships");

    if membership.len() > 0 {
        Ok(membership)
    } else {
        Err(NoResultInDatabase)
    }
}

/// Public function used to return all memberships linked to a certain channel_id,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_all_channel_memberships(connection, 12);
/// ```
pub fn get_all_channel_memberships(connection: &mut MysqlConnection, w_id: i32) -> Result<Vec<Membership>, Error> {
    use crate::rirc_schema::memberships::dsl::*;

    let mut membership = memberships
        .filter(id_channel.eq(w_id))
        .load::<Membership>(connection)
        .expect("Error loading memberships");

    if membership.len() > 0 {
        Ok(membership)
    } else {
        Err(NoResultInDatabase)
    }
}

/// Public function used to create memberships
pub fn create_membership(connection: &mut MysqlConnection, nick: &str, channel: &str) {
    use crate::rirc_schema::memberships;

    let id_user = &get_user(connection, nick).unwrap().id;
    let id_channel = &get_channel(connection, channel).unwrap().id;

    let new_membership = NewMembership { id_user, id_channel };

    diesel::insert_into(memberships::table)
        .values(&new_membership)
        .execute(connection)
        .expect("Error saving new membership");
}

pub fn delete_user_membership(connection: &mut MysqlConnection, w_nick: &str) {
    use crate::rirc_schema::memberships;
    use crate::rirc_schema::memberships::dsl::*;

    let user_id = get_user(connection, w_nick).unwrap().id;

    diesel::delete(memberships::table)
        .filter(id_user.eq(user_id))
        .execute(connection)
        .expect("Error removing memberships");
}

pub fn delete_membership(connection: &mut MysqlConnection, w_id: i32) {
    use crate::rirc_schema::memberships;
    use crate::rirc_schema::memberships::dsl::*;

    diesel::delete(memberships::table)
        .filter(id.eq(w_id))
        .execute(connection)
        .expect("Error removing memberships");
}

/// Returns only the first word of the given `str`.
pub fn first_word(content: &str) -> &str {
    content.split_whitespace().next().unwrap_or(&*content)
}

/// Public struct used to hold IP and port to listen to,
///
/// Example:
/// ```rust
/// let server = Server::new("127.0.0.1", 6667);
/// let socket = SocketAddr::new(server.get_addr(), server.get_port());
/// ```
#[derive(Clone, Copy)]
pub struct Server {
    addr: IpAddr,
    port: u16,
}

#[allow(dead_code)]
impl Server {
    /// Turns a `&str` and `u16` into a `Server`.
    ///
    /// Example: `Server::new("127.0.0.1", 6667);`.
    pub fn new(addr: &str, port: u16) -> Server {
        return Server {
            addr: Server::parse_addr(addr),
            port,
        };
    }

    /// Public function creating a Server from two Settings
    ///
    /// Example:
    /// ```rust
    /// let connection = &mut establish_connection();
    /// let server = Server::from_settings(get_setting(connection, "ip"), get_setting(connection, "port"));
    /// ```
    pub fn from_settings(addr: Setting, port: Setting) -> Server {
        return Server::new(
            addr.content.as_str(),
            port.content.parse().unwrap()
        )
    }

    /// This function is used to parse IPv4 `&str` into `IpAddr::V4`.
    ///
    /// Example: `parse_addr("127.0.0.1")`.
    fn parse_addr(addr: &str) -> IpAddr {
        // Split str argument into Vec<&str>
        let split_addr: Vec<&str> = addr.split(".").collect();

        // Stupido checks
        let mut panic = false;
        if split_addr.clone().len() > 4 {
            panic = true;
        };
        for i in split_addr.clone() {
            let j = i.parse::<u32>();

            if j.is_err() {
                panic = true
            }
            if j.unwrap() > 254 {
                panic = true;
            };
        }

        if panic {
            panic!("Given IP address for server seems invalid.");
        };

        // Use splits to build IpAddr
        return IpAddr::V4(Ipv4Addr::new(
            split_addr[0].to_string().parse().unwrap(),
            split_addr[1].to_string().parse().unwrap(),
            split_addr[2].to_string().parse().unwrap(),
            split_addr[3].to_string().parse().unwrap(),
        ));
    }

    pub fn get_addr(self) -> IpAddr {
        return self.addr;
    }
    pub fn get_port(self) -> u16 {
        return self.port;
    }

    pub fn set_addr(&mut self, addr: &str) {
        return self.addr = Server::parse_addr(addr);
    }
    pub fn set_port(&mut self, port: u16) {
        return self.port = port;
    }
}
