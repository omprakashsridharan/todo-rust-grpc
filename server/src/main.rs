mod db;
mod interceptors;
mod service_impl;

use crate::db::{get_connection_pool, Manager, Message};
use crate::interceptors::AuthInterceptor;
use crate::service_impl::{AuthService, TodoService};
use dotenv::dotenv;
use proto::service::auth::auth_server::AuthServer;
use proto::service::todo::todo_server::TodoServer;
use std::env;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();
    dotenv().ok();

    // Database Manager setup
    let pool = get_connection_pool().await?;
    let (db_tx, db_rx) = tokio::sync::mpsc::channel::<Message>(32);
    tokio::spawn(async move {
        let mut manager = Manager::new(pool, db_rx);
        manager.listen().await;
    });

    // Middleware manager
    let auth_interceptor = AuthInterceptor::default();

    let port = env::var("PORT").unwrap_or(String::from("50050"));
    // Address
    let adder = format!("0.0.0.0:{}", port).parse()?;
    info!("Server running on {:?}", adder);
    // Initiate service defaults
    let auth_service = AuthService::new(db_tx.clone());
    let todo_service = TodoService::new(db_tx.clone());

    let auth_service = AuthServer::new(auth_service);
    let todo_service_with_interceptor =
        TodoServer::with_interceptor(todo_service, auth_interceptor);

    Server::builder()
        .add_service(auth_service)
        .add_service(todo_service_with_interceptor)
        .serve(adder)
        .await?;
    Ok(())
}
