use once_cell::sync::OnceCell;
use schoology::SchoologyClient;

static CLIENT: OnceCell<SchoologyClient> = OnceCell::new();

pub fn get_schoology_client() -> &'static SchoologyClient {
    CLIENT.get().expect("Database client not initialized")
}

pub async fn create_schoology_client(
    consumer_key: String,
    consumer_secret: String,
) -> Result<(), String> {
    info!("Creating Schoology client...");

    let client = SchoologyClient::new(consumer_key, consumer_secret);

    // Set the schoology client
    CLIENT
        .set(client)
        .map_err(|_| "Failed to set Schoology client".to_string())
}
