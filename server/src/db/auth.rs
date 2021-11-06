use sqlx::FromRow;

#[derive(Debug, FromRow, Clone)]
pub struct User {
    pub username: String,
    pub pin: i32,
}
