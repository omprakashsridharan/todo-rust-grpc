use crate::db::models::User;
use crate::service::auth::SignUpRequest;
use sqlx::mysql::MySqlDatabaseError;
use sqlx::{pool::PoolConnection, MySql, Pool};
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
}

pub struct Manager {
    pool: Pool<MySql>,
    receiver: Receiver<Message>,
}

impl Manager {
    pub fn new(pool: Pool<MySql>, receiver: Receiver<Message>) -> Self {
        Self { pool, receiver }
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
                    error!("Some other error while inserting into database");
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
                        Err(e) => eprintln!("Unable to send back Sign up manager {:?}", e),
                    }
                }
            }
        }
    }
}
