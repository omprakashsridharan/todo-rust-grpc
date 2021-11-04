use crate::db::models::User;
use crate::service::auth::{SignInRequest, SignUpRequest};
use hmac::{Hmac, NewMac};
use jwt::SignWithKey;
use sha2::Sha256;
use sqlx::mysql::MySqlDatabaseError;
use sqlx::{pool::PoolConnection, MySql, Pool};
use std::collections::BTreeMap;
use std::env;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender as OneShotSender;
use tracing::info;
use tracing::log::error;

#[derive(Debug)]
pub enum Message {
    SignUp {
        req: SignUpRequest,
        resp: OneShotSender<Result<User, String>>,
    },
    SignIn {
        req: SignInRequest,
        resp: OneShotSender<Result<String, String>>,
    },
}

pub struct Manager {
    pool: Pool<MySql>,
    receiver: Receiver<Message>,
}

fn generate_jwt(username: String) -> String {
    let secret = env::var("SECRET").expect("SECRET env missing");
    let key: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes()).unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("sub", &username);
    let token_str = claims.sign_with_key(&key).unwrap();
    return token_str;
}

impl Manager {
    pub fn new(pool: Pool<MySql>, receiver: Receiver<Message>) -> Self {
        Self { pool, receiver }
    }

    async fn sign_in(
        conn: &mut PoolConnection<MySql>,
        req: SignInRequest,
    ) -> Result<String, String> {
        let result = sqlx::query_as!(
            User,
            "select username,pin from user where username = ? and pin = ?",
            req.username.clone(),
            req.pin.clone()
        )
        .fetch_one(conn)
        .await;

        match result {
            Ok(mysql_result) => Ok(generate_jwt(mysql_result.username)),
            Err(e) => match e {
                sqlx::Error::Database(db_err) => {
                    error!("Database error {:?}", db_err);
                    let mysql_error = db_err.downcast::<MySqlDatabaseError>();
                    let message = (*mysql_error).message().to_string();
                    Err(message)
                }
                _ => {
                    error!("Some other error while getting from database {:?}", e);
                    Err(String::from("Error while signing in"))
                }
            },
        }
    }

    async fn sign_up(conn: &mut PoolConnection<MySql>, req: SignUpRequest) -> Result<User, String> {
        let result = sqlx::query("INSERT into user (username, pin) VALUES (?, ?)")
            .bind(req.username.clone())
            .bind(req.pin.clone())
            .execute(conn)
            .await;
        match result {
            Ok(mysql_result) => {
                info!("Sign up result is {:?}", mysql_result);
                Ok(User {
                    username: req.username.clone(),
                    pin: req.pin.clone(),
                })
            }
            Err(e) => match e {
                sqlx::Error::Database(db_err) => {
                    error!("Database error {:?}", db_err);
                    let mysql_error = db_err.downcast::<MySqlDatabaseError>();
                    let message = (*mysql_error).message().to_string();
                    Err(message)
                }
                _ => {
                    error!("Some other error while inserting into database {:?}", e);
                    Err(String::from("Error while inserting user into database"))
                }
            },
        }
    }

    pub async fn listen(&mut self) {
        let mut connection = self.pool.acquire().await.unwrap();
        while let Some(message) = self.receiver.recv().await {
            match message {
                Message::SignUp { req, resp } => {
                    let sign_up_result = Self::sign_up(&mut connection, req).await;
                    match resp.send(sign_up_result) {
                        Ok(_) => {}
                        Err(e) => error!("Unable to send back from Sign up manager {:?}", e),
                    }
                }
                Message::SignIn { req, resp } => {
                    let sign_in_result = Self::sign_in(&mut connection, req).await;
                    match resp.send(sign_in_result) {
                        Ok(_) => {}
                        Err(e) => error!("Unable to send back from Sign in manager {:?}", e),
                    }
                }
            }
        }
    }
}
