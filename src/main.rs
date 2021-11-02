mod service;
mod service_impl;

use crate::service::auth::auth_server::AuthServer;
use crate::service_impl::AuthService;
use dotenv::dotenv;
use std::env;
use tonic::transport::Server;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let port = env::var("PORT").unwrap_or(String::from("50051"));
    // Address
    let adder = format!("[::1]:{}", port).parse()?;

    // Initiate service defaults
    let auth_service = AuthService::default();

    Server::builder()
        .add_service(AuthServer::new(auth_service))
        .serve(adder)
        .await?;
    Ok(())
}
