mod db;
mod interceptors;
mod service;
mod service_impl;

use crate::db::{get_connection_pool, Manager, Message};
use crate::interceptors::AuthInterceptor;
use crate::service::auth::auth_server::AuthServer;
use crate::service::todo::todo_server::TodoServer;
use crate::service_impl::{AuthService, TodoService};
use dotenv::dotenv;
use std::env;
use tonic::transport::Server;
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
    let _auth_interceptor = AuthInterceptor::default();

    let port = env::var("PORT").unwrap_or(String::from("50051"));
    // Address
    let adder = format!("[::1]:{}", port).parse()?;

    // Initiate service defaults
    let auth_service = AuthService::new(db_tx.clone());
    let todo_service = TodoService::new(db_tx.clone());

    let auth_service_with_interceptor = AuthServer::new(auth_service);
    Server::builder()
        .add_service(auth_service_with_interceptor)
        .add_service(TodoServer::new(todo_service))
        .serve(adder)
        .await?;
    Ok(())
}
