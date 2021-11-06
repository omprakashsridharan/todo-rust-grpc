use crate::db::models::User;
use crate::db::Message;
use proto::service::auth::auth_server::Auth;
use proto::service::auth::{SignInRequest, SignInResponse, SignUpRequest, SignUpResponse};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::channel;
use tonic::{Request, Response, Status};
use tracing::{error, info};

#[derive(Debug, Clone)]
pub struct AuthService {
    db_message_sender: Sender<Message>,
}

impl AuthService {
    pub fn new(db_message_sender: Sender<Message>) -> Self {
        Self { db_message_sender }
    }
}

#[tonic::async_trait]
impl Auth for AuthService {
    async fn sign_up(
        &self,
        request: Request<SignUpRequest>,
    ) -> Result<Response<SignUpResponse>, Status> {
        if request.get_ref().pin > 9999 || request.get_ref().pin < 1000 {
            let error_message = format!(
                "Sign Up for Username: {} - PIN should consist only 4 digits",
                request.get_ref().username.to_string()
            );
            error!("{}", error_message);
            return Err(Status::invalid_argument(error_message));
        }
        let (tx, rx) = channel::<Result<User, String>>();
        match self
            .db_message_sender
            .send(Message::SignUp {
                req: request.get_ref().clone(),
                resp: tx,
            })
            .await
        {
            Ok(_) => {}
            Err(e) => error!("Failed to send sign up message to DB manager {:?}", e),
        }
        match rx.await {
            Ok(res) => match res {
                Ok(user) => {
                    info!("Signed up {:?}", user);
                    let reply = SignUpResponse {
                        message: "Signed up successfully".into(),
                        success: true,
                    };
                    Ok(Response::new(reply))
                }
                Err(e) => {
                    error!("Error while signing up {:?}", e);
                    Err(Status::aborted(format!("Error while signing up: {}", e)))
                }
            },
            Err(e) => {
                error!("Error while signing up {:?}", e);
                Err(Status::aborted("Error while signing up"))
            }
        }
    }

    async fn sign_in(
        &self,
        request: Request<SignInRequest>,
    ) -> Result<Response<SignInResponse>, Status> {
        let (tx, rx) = channel::<Result<String, String>>();
        match self
            .db_message_sender
            .send(Message::SignIn {
                req: request.get_ref().clone(),
                resp: tx,
            })
            .await
        {
            Ok(_) => {}
            Err(e) => error!("Failed to send sign in message to DB manager {:?}", e),
        }
        match rx.await {
            Ok(res) => match res {
                Ok(token) => {
                    info!("Signed in user: {}", request.get_ref().username);
                    let reply = SignInResponse { token };
                    Ok(Response::new(reply))
                }
                Err(e) => {
                    error!("Error while signing in {:?}", e);
                    Err(Status::unauthenticated(e))
                }
            },
            Err(e) => {
                error!("Error while signing up {:?}", e);
                Err(Status::aborted("Error while signing up"))
            }
        }
    }
}
