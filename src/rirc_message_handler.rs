use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;
use diesel::MysqlConnection;
use log::trace;
use crate::rirc_conn_handler::sender;
use crate::rirc_lib::*;

pub fn wait_for_message(connection: &mut MysqlConnection, mut stream: TcpStream) {
    // fetch last membership in db (so we know whats our purpose)
    let membership = get_last_membership(connection).unwrap();

    // fetch corresponding user and channels
    let user = get_user_from_id(connection, &membership.id_user).unwrap();
    let channel = get_channel_from_id(connection, &membership.id_channel).unwrap();

    // store channel's last message
    let mut message = channel.content;

    // store who this owns this thread
    let owner = user.nick.as_str();

    loop {
        trace!("oui");
        // get channel's last message
        let new_message = get_channel_from_id(connection, &membership.id_channel).unwrap().content;

        // if message is not new, ignore
        if new_message == message {
            continue
        }

        // if message is sent by thread owner, ignore
        if message.starts_with(&(":".to_string() + owner)) {
            message = new_message;

            continue
        }

        let res = Response::new(message);
        sender(stream.try_clone().unwrap(), res);

        // TODO: if channel 's content is user leaving, close thread

        message = new_message;

        sleep(Duration::from_millis(500000))
    }
}