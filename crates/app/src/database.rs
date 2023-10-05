use once_cell::sync::OnceCell;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

static CLIENT: OnceCell<DatabaseConnection> = OnceCell::new();

pub fn get_db_client() -> &'static DatabaseConnection {
    CLIENT.get().expect("Database client not initialized")
}

pub async fn create_db_client(
    db_url: &str,
    max_connections: usize,
    min_connections: usize,
    connect_timeout: usize,
) -> Result<(), String> {
    info!(
        "Attempting to connect to database.. (timeout: {}s)",
        connect_timeout
    );

    let mut options = ConnectOptions::new(db_url);

    options
        .max_connections(max_connections as u32)
        .min_connections(min_connections as u32)
        .connect_timeout(std::time::Duration::from_secs(connect_timeout as u64))
        .sqlx_logging(true);

    let db = Database::connect(options)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;

    info!("Connected to database!");

    // Set the database client
    CLIENT
        .set(db)
        .map_err(|_| "Failed to set database client".to_string())
}
