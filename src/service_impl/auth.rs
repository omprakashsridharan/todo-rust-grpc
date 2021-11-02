use crate::service::auth::auth_server::Auth;
use crate::service::auth::{SignUpRequest, SignUpResponse};
use tonic::{Request, Response, Status};
use tracing::{error, info};

#[derive(Debug, Default)]
pub struct AuthService {}

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
        info!("Signed up");
        let reply = SignUpResponse {
            message: "Signed up successfully".into(),
            success: true,
        };
        Ok(Response::new(reply))
    }
}
