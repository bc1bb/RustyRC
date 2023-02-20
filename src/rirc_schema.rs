// @generated automatically by Diesel CLI.

diesel::table! {
    bans (id) {
        id -> Integer,
        is_ip -> Bool,
        content -> Char,
    }
}

diesel::table! {
    channels (id) {
        id -> Integer,
        name -> Char,
        creation_time -> Integer,
        creator -> Char,
        topic -> Mediumtext,
        content -> Longtext,
    }
}

diesel::table! {
    memberships (id) {
        id -> Integer,
        id_user -> Integer,
        id_channel -> Integer,
    }
}

diesel::table! {
    settings (id) {
        id -> Integer,
        key -> Char,
        content -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        last_login -> Bigint,
        nick -> Char,
        real_name -> Char,
        last_ip -> Char,
        is_connected -> Bool,
        op -> Bool,
        thread_id -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    bans,
    channels,
    memberships,
    settings,
    users,
);
