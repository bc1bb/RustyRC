mod rirc_conn_handler;
mod rirc_schema;
mod rirc_lib;
mod rirc_protocol_handler;

use std::net::{SocketAddr, TcpListener};
use std::thread::spawn;
use dotenvy::dotenv;
use log::{debug, info};
use crate::rirc_lib::*;
use crate::rirc_conn_handler::handler;

/// Main function, holds threads, database connection
fn main() {
    env_logger::init();
    dotenv().ok();

    debug!("Connecting to database...");
    let connection = &mut establish_connection();
    clean_database(connection);

    // This gets settings from database to create a `Server`.
    let server = Server::from_settings(get_setting(connection, "ip").unwrap(), get_setting(connection, "port").unwrap());

    let socket = SocketAddr::new(server.get_addr(), server.get_port());

    info!("Starting listener on {}:{}", server.get_addr(), server.get_port());
    let listener = TcpListener::bind(socket).unwrap();

    debug!("Starting connection manager...");
    // Spawning a thread of handler() for each incoming connection
    for (thread_id, stream) in listener.incoming().enumerate() {
        spawn(move || {
            let connection = &mut establish_connection();

            let addr = stream.as_ref().unwrap().peer_addr().unwrap();
            debug!("New connection from {}", addr);

            handler(connection, stream.unwrap(), i32::try_from(thread_id).unwrap());
        });
    }
}