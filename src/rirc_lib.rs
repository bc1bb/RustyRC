//! RustyIRC Lib
//!
//! Shared file containing different structs and public functions for other modules to work.
//!
//! Including objects used for database communication.

use std::env;
use std::net::{IpAddr, Ipv4Addr};
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use dotenvy::dotenv;
use super::rirc_schema::*;

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
#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub last_ip: String,
    pub is_connected: bool,
}

/// Insertable public struct linked to database using Diesel.
#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub name: &'a str,
    pub last_ip: &'a str,
    pub is_connected: &'a bool,
}

/// Public function that will return a `User` when given its `name`,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_user(connection, "johndoe");
/// ```
pub fn get_user<'a>(connection: &mut MysqlConnection,  w_name: &str) -> User {
    use crate::rirc_schema::users::dsl::*;

    users
        .limit(1)
        .filter(name.eq(w_name))
        .load::<User>(connection)
        .expect("Error loading users")
        .into_iter()
        .nth(0)
        .unwrap()
}

/// Public function that handles creating users,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// create_user(connection, "johndoe", "1.2.3.4", &true);
/// ```
pub fn create_user(connection: &mut MysqlConnection, name: &str,
                   last_ip: &str, is_connected: &bool) {
    use crate::rirc_schema::users;

    let new_user = NewUser { name, last_ip, is_connected };

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(connection)
        .expect("Error saving new user");
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
/// get_channel(connection, &true, "1.1.1.1");
/// get_channel(connection, &false, "johndoe");
/// ```
pub fn get_ban<'a>(connection: &mut MysqlConnection, w_is_ip: &bool, w_name: &str) -> Ban {
    use crate::rirc_schema::bans::dsl::*;

    bans
        .limit(1)
        .filter(is_ip.eq(w_is_ip))
        .filter(content.eq(w_name))
        .load::<Ban>(connection)
        .expect("Error loading bans")
        .into_iter()
        .nth(0)
        .unwrap()
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
pub fn get_channel<'a>(connection: &mut MysqlConnection, w_name: &str) -> Channel {
    use crate::rirc_schema::channels::dsl::*;

    channels
        .limit(1)
        .filter(name.eq(w_name))
        .load::<Channel>(connection)
        .expect("Error loading channels")
        .into_iter()
        .nth(0)
        .unwrap()
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

/// Queryable public struct linked to database using Diesel.
#[derive(Queryable)]
pub struct Setting {
    pub id: i32,
    pub key: String,
    pub content: String,
}

/// Insertable struct linked to database using Diesel.
#[derive(Insertable)]
#[diesel(table_name = settings)]
struct NewSetting<'a> {
    pub key: &'a str,
    pub content: &'a str,
}

/// Public function that will return a `Setting` when given it's `key`
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// get_setting(connection, "name");
/// ```
pub fn get_setting<'a>(connection: &mut MysqlConnection, w_key: &str) -> Setting {
    use crate::rirc_schema::settings::dsl::*;

    settings
        .limit(1)
        .filter(key.eq(w_key))
        .load::<Setting>(connection)
        .expect("Error loading settings")
        .into_iter()
        .nth(0)
        .unwrap()
}

/// Public function that handles creating settings,
///
/// Example:
/// ```rust
/// let connection = &mut establish_connection();
/// create_setting(connection, "name", "MyCoolServ");
/// ```
pub fn create_setting(connection: &mut MysqlConnection, key: &str, content: &str) {
    use crate::rirc_schema::settings;

    let new_setting = NewSetting { key, content };

    diesel::insert_into(settings::table)
        .values(&new_setting)
        .execute(connection)
        .expect("Error saving new setting");
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
