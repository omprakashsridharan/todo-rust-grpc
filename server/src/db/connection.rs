use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySql, Pool};
use std::env;

pub async fn get_connection_pool() -> Result<Pool<MySql>, sqlx::Error> {
    let database_uri = env::var("DATABASE_URL").expect("DATABASE_URL is missing");
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_uri)
        .await;
    return pool;
}
