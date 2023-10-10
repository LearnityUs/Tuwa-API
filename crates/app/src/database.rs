use once_cell::sync::OnceCell;
use orm::{schoology_request_tokens, sessions};
use sea_orm::{
    ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait, QueryFilter,
};

use crate::v1::schoology;

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

pub async fn cronjob_clear_old() {
    info!("Clearing expired Schoology request tokens...");

    let db_client = get_db_client();

    // Delere all the expired tokens
    let result = schoology_request_tokens::Entity::delete_many()
        .filter(schoology_request_tokens::Column::ExpiresAt.lt(chrono::Utc::now()))
        .exec(db_client)
        .await;

    if let Err(err) = result {
        error!(
            "Failed to delete expired Schoology request tokens: {:?}",
            err
        );
    }

    info!("Clearing expired sessions...");

    let result = sessions::Entity::delete_many()
        .filter(sessions::Column::ExpiresAt.lt(chrono::Utc::now()))
        .exec(db_client)
        .await;

    if let Err(err) = result {
        error!("Failed to delete expired sessions: {:?}", err);
    }
}
