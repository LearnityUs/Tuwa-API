#[macro_use]
extern crate log;
use actix_cors::Cors;
use actix_web::{
    dev::Service,
    http::{self, header},
    middleware, web, App, HttpResponse, HttpServer,
};
use chrono::Duration;
use glob_match::glob_match;
use tokio::signal::unix::SignalKind;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::{
    database::{create_db_client, cronjob_clear_old},
    schoology::create_schoology_client,
    v1::create_v1_service,
};

mod database;
mod schoology;
pub mod utils;
mod v1;

async fn not_found() -> actix_web::HttpResponse {
    HttpResponse::NotFound()
        .content_type(http::header::ContentType::plaintext())
        .body("Rawr 🦖! This page was not found!")
}

async fn server() -> Result<(), String> {
    // Get the port from the environment
    let port = std::env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse::<u16>()
        .unwrap_or(5000);

    info!("Starting server at [::1]:{}", port);

    let server = HttpServer::new(|| {
        App::new()
            .service(web::scope("/api").service(create_v1_service()))
            .default_service(web::route().to(not_found))
            // Strip the `Forwarded` header cause GCP uses `X-Forwarded-For`. See: https://cloud.google.com/load-balancing/docs/https
            // This prevents ip spoofing
            .wrap_fn(|req, srv| {
                let mut req = req;

                req.headers_mut().remove(header::FORWARDED);

                srv.call(req)
            })
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(Cors::default().allowed_origin_fn(|origin, _re_head| {
                // Match the glob for cors origins
                if let (Ok(cors_origin), Ok(origin_str)) =
                    (std::env::var("CORS_ORIGIN"), origin.to_str())
                {
                    glob_match(&cors_origin, origin_str)
                } else {
                    // Better safe then sorry
                    false
                }
            }))
    })
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

    // Database stuff
    let max_connections = std::env::var("DB_MAX_CONNECTIONS")
        .unwrap_or("10".to_string())
        .parse::<usize>()
        .unwrap_or(10);

    let min_connections = std::env::var("DB_MIN_CONNECTIONS")
        .unwrap_or("1".to_string())
        .parse::<usize>()
        .unwrap_or(1);

    let connect_timeout = std::env::var("DB_CONNECT_TIMEOUT")
        .unwrap_or("10".to_string())
        .parse::<usize>()
        .unwrap_or(10);

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    match create_db_client(&db_url, max_connections, min_connections, connect_timeout).await {
        Ok(_) => info!("Database client created"),
        Err(e) => {
            error!("Failed to create database client: {}", e);
            std::process::exit(1);
        }
    };

    // Schoology stuff
    let consumer_key =
        std::env::var("SCHOOLOGY_CONSUMER_KEY").expect("SCHOOLOGY_CONSUMER_KEY must be set");
    let consumer_secret =
        std::env::var("SCHOOLOGY_CONSUMER_SECRET").expect("SCHOOLOGY_CONSUMER_SECRET must be set");

    match create_schoology_client(consumer_key, consumer_secret).await {
        Ok(_) => info!("Schoology client created"),
        Err(e) => {
            error!("Failed to create Schoology client: {}", e);
            std::process::exit(1);
        }
    }

    // Cronjob stuff
    let scheduler = match JobScheduler::new().await {
        Ok(scheduler) => scheduler,
        Err(e) => {
            error!("Failed to create scheduler: {}", e);
            std::process::exit(1);
        }
    };

    // Every 5 minutes
    let job =
        match Job::new_repeated_async(Duration::minutes(5).to_std().unwrap(), |_uuid, _lock| {
            Box::pin(async {
                cronjob_clear_old().await;
            })
        }) {
            Ok(job) => job,
            Err(e) => {
                error!("Failed to create job: {}", e);
                std::process::exit(1);
            }
        };

    match scheduler.add(job).await {
        Ok(_) => info!("Job added"),
        Err(e) => {
            error!("Failed to add job: {}", e);
            std::process::exit(1);
        }
    };

    // Shutdown on SIGINT and SIGTERM and SIGQUIT
    scheduler.shutdown_on_signal(SignalKind::terminate());
    scheduler.shutdown_on_signal(SignalKind::interrupt());
    scheduler.shutdown_on_signal(SignalKind::quit());

    // Start the job scheduler
    match scheduler.start().await {
        Ok(_) => info!("Scheduler stopped"),
        Err(e) => error!("Scheduler stopped with error: {}", e),
    };

    match server().await {
        Ok(_) => info!("Server stopped"),
        Err(e) => error!("Server stopped with error: {}", e),
    };
}
