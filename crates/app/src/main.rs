#[macro_use]
extern crate log;
use actix_web::{get, web, App, HttpServer, Responder};

use crate::v1::create_v1_service;

mod v1;

async fn server() -> Result<(), String> {
    // Get the port from the environment
    let port = std::env::var("PORT").unwrap_or("5000".to_string());
    let port = port.parse::<u16>().unwrap_or(5000);

    // Get the cors allow site from the environment
    let cors_headers = std::env::var("CORS_SITE").ok();

    info!("Starting server on port {}", port);

    let server =
        HttpServer::new(|| App::new().service(web::scope("/api").service(create_v1_service())))
            .bind(("127.0.0.1", port))
            .map_err(|e| format!("Failed to bind server: {}", e))?
            .run();

    server.await.map_err(|e| format!("Server failed: {}", e))?;

    Ok(())
}

#[tokio::main]
async fn main() {
    // Init dotenv
    #[cfg(debug_assertions)]
    dotenv::dotenv().ok();

    pretty_env_logger::init();

    match server().await {
        Ok(_) => info!("Server stopped"),
        Err(e) => error!("Server stopped with error: {}", e),
    };
}
