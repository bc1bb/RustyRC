//! @generated automatically by Diesel CLI.

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
        motd -> Mediumtext,
        content -> Longtext,
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
        name -> Char,
        last_ip -> Char,
        is_connected -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    bans,
    channels,
    settings,
    users,
);
