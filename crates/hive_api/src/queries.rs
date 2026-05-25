//! SQL query text loaded once at compile time and stored in the binary.
//!
//! Each constant is a `&'static str`, so there is no runtime file I/O or caching
//! layer to maintain — access is a direct pointer read.

macro_rules! sql {
    ($path:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/queries/", $path))
    };
}

pub mod users {
    pub const INSERT_USER: &str = sql!("users/insert_user.sql");
    pub const FIND_USER_BY_EMAIL: &str = sql!("users/find_user_by_email.sql");
    pub const UPDATE_USER_LAST_LOGIN: &str = sql!("users/update_user_last_login.sql");
    pub const REVOKE_SESSION: &str = sql!("users/revoke_session.sql");
    pub const INSERT_SESSION: &str = sql!("users/insert_session.sql");
    pub const AUTHENTICATE_SESSION: &str = sql!("users/authenticate_session.sql");
}

pub mod games {
    pub const INSERT_WAITING_GAME: &str = sql!("games/insert_waiting_game.sql");
    pub const FIND_WAITING_GAME_BY_INVITE: &str = sql!("games/find_waiting_game_by_invite.sql");
    pub const FIND_GAME_BY_INVITE: &str = sql!("games/find_game_by_invite.sql");
    pub const FIND_GAME_BY_ID: &str = sql!("games/find_game_by_id.sql");
    pub const FIND_GAME_BY_ID_FOR_UPDATE: &str = sql!("games/find_game_by_id_for_update.sql");
    pub const JOIN_WAITING_GAME: &str = sql!("games/join_waiting_game.sql");
    pub const LIST_USER_GAMES: &str = sql!("games/list_user_games.sql");
    pub const LIST_ACTIONS_FOR_GAME: &str = sql!("games/list_actions_for_game.sql");
    pub const INSERT_SOLO_GAME: &str = sql!("games/insert_solo_game.sql");
    pub const INSERT_ACTION: &str = sql!("games/insert_action.sql");
    pub const UPDATE_GAME_STATUS: &str = sql!("games/update_game_status.sql");
}
