mod rirc_conn_handler;
mod rirc_schema;
mod rirc_lib;

use std::net::{IpAddr, SocketAddr, TcpListener};
use std::thread::spawn;
use dotenvy::dotenv;
use log::{debug, info, trace, warn};
use crate::rirc_lib::{create_user, establish_connection, get_setting, Server};
use crate::rirc_conn_handler::handler;

/// Main function, holds threads, database connection
fn main() {
    env_logger::init();
    dotenv().ok();

    debug!("Connecting to database...");
    let connection = &mut establish_connection();

    // This gets settings from database to create a `Server`.
    let server = Server::from_settings(get_setting(connection, "ip"), get_setting(connection, "port"));

    let socket = SocketAddr::new(server.get_addr(), server.get_port());

    info!("Starting listener on {}:{}", server.get_addr(), server.get_port());
    let listener = TcpListener::bind(socket).unwrap();

    debug!("Starting connection manager...");
    // Spawning a thread of handler() for each incoming connection
    for stream in listener.incoming() {
        spawn(|| {
            debug!("New connection from {}", stream.as_ref().unwrap().peer_addr().unwrap());

            handler(stream.unwrap());
            // TODO: Return a user from handler() and change its state on DB when it logs off
        });
    }
}