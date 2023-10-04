#[macro_use]
extern crate log;
use actix_web::{web, App, HttpServer, HttpResponse, http};

use crate::v1::create_v1_service;

mod v1;

async fn not_found() -> actix_web::HttpResponse {
    HttpResponse::BadRequest().content_type(http::header::ContentType::plaintext()).body("Rarw! This page was not found!")
}

async fn server() -> Result<(), String> {
    // Get the port from the environment
    let port = std::env::var("PORT").unwrap_or("8080".to_string());
    let port = port.parse::<u16>().unwrap_or(5000);

    // Get the cors allow site from the environment
    // let cors_headers = std::env::var("CORS_SITE").ok();

    info!("Starting server on port {}", port);

    let server =
        HttpServer::new(|| App::new()
            .service(
                web::scope("/api")
                    .service(create_v1_service())
            )
            .default_service(web::route().to(not_found))
        )
            .bind(("0.0.0.0", port))
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
