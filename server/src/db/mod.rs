mod auth;
mod connection;
mod manager;
mod todo;

pub use crate::db::connection::get_connection_pool;
pub use crate::db::manager::{Manager, Message};
pub mod models {
    pub use crate::db::auth::User;
    pub use crate::db::todo::TodoItemDb;
}
