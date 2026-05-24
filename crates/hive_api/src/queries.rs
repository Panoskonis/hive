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
