use hmac::{Hmac, NewMac};
use jwt::VerifyWithKey;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::env;
use tonic::{service::Interceptor, Request, Status};
use tracing::log::{error, info};

pub struct AuthExtension {
    pub username: String,
}
#[derive(Clone, Default, Debug)]
pub struct AuthInterceptor {}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        match request.metadata().get("authorization") {
            Some(t) => match (*t).to_str() {
                Ok(full_token) => {
                    let secret = env::var("SECRET").expect("SECRET env missing");
                    let key: Hmac<Sha256> = Hmac::new_from_slice(secret.as_bytes()).unwrap();
                    let claims: BTreeMap<String, String> =
                        full_token.verify_with_key(&key).unwrap();
                    info!("Claims token {:?}", claims);
                    let username: String = claims["sub"].to_owned();
                    request.extensions_mut().insert(AuthExtension { username });

                    return Ok(request);
                }
                Err(e) => {
                    error!("Error while parsing token {:?}", e);
                    return Err(Status::unauthenticated("No valid auth token"));
                }
            },
            _ => return Err(Status::unauthenticated("No valid auth token")),
        }
    }
}
