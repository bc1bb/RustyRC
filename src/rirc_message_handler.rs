use std::net::TcpStream;
use std::time::Duration;
use diesel::MysqlConnection;
use log::trace;
use spin_sleep::LoopHelper;

use crate::rirc_conn_handler::sender;
use crate::rirc_lib::*;

pub fn wait_for_message(connection: &mut MysqlConnection, mut stream: TcpStream) {
    // Using spin_sleep::LoopHelper to build a loop
    let mut loop_helper = LoopHelper::builder()
        .report_interval_s(0.5)
        .build_with_target_rate(0.5); // Every half a second

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
        loop_helper.loop_start();

        // get channel's last message
        let new_message = get_channel_from_id(connection, &membership.id_channel).unwrap().content;

        // if message is not new, ignore
        if new_message == message {
            // Sleeps
            loop_helper.loop_sleep();

            continue
        }

        // if message is sent by thread owner, ignore
        if message.starts_with(&(":".to_string() + owner)) {
            message = new_message;

            // Sleeps
            loop_helper.loop_sleep();

            continue
        }

        let res = Response::new(message);
        sender(stream.try_clone().unwrap(), res);

        // TODO: if channel 's content is user leaving, close thread

        message = new_message;

        // Sleeps
        loop_helper.loop_sleep();
    }
}