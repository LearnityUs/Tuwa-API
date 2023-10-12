use base64::{engine::general_purpose, Engine};
use hmac::{Hmac, Mac};
use orm::schoology_request_tokens;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use sha2::Sha512;
use uuid::Uuid;

/// Creates a signature for a request token
pub fn sign(uuid: Uuid, token_secret: String) -> Result<String, ()> {
    // Hash the uuid with the signature
    let hash = match Hmac::<Sha512>::new_from_slice(token_secret.as_bytes()) {
        Ok(mut hmac) => {
            hmac.update(uuid.as_bytes());
            hmac.finalize()
        }
        Err(err) => {
            error!("Failed to hash request token: {:?}", err);
            return Err(());
        }
    };

    // Base64 encode the hash
    let hash = general_purpose::STANDARD_NO_PAD.encode(&hash.into_bytes());

    Ok(hash)
}

/// Gets a request token from the database
pub async fn get(
    db_connection: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<schoology_request_tokens::Model>, ()> {
    // Query the database
    match schoology_request_tokens::Entity::find_by_id(id)
        .one(db_connection)
        .await
        .map_err(|err| {
            debug!("Failed to get schoology request token: {:?}", err);
        })? {
        Some(schoology_request_token) => Ok(Some(schoology_request_token)),
        None => return Ok(None),
    }
}

/// Creates a request token in the database
pub async fn create(
    db_connection: &DatabaseConnection,
    access_token: String,
    token_secret: String,
    ttl: usize,
) -> Result<schoology_request_tokens::Model, ()> {
    // Create a uuid
    let uuid = Uuid::new_v4();

    // Create timestamp
    let expires_at = chrono::Utc::now() + chrono::Duration::seconds(ttl as i64);

    // Create the request token
    let schoology_request_token = schoology_request_tokens::ActiveModel {
        id: ActiveValue::Set(uuid),
        access_token: ActiveValue::Set(access_token),
        token_secret: ActiveValue::Set(token_secret),
        expires_at: ActiveValue::Set(expires_at.naive_utc()),
    };

    // Insert the request token into the database
    let schoology_request_token = schoology_request_token.insert(db_connection).await;

    match schoology_request_token {
        Ok(schoology_request_token) => Ok(schoology_request_token),
        Err(err) => {
            warn!("Failed to create schoology request token: {:?}", err);
            Err(())
        }
    }
}

/// Deletes a request token from the database
pub async fn delete(db_connection: &DatabaseConnection, id: Uuid) -> Result<(), ()> {
    schoology_request_tokens::Entity::delete_by_id(id)
        .exec(db_connection)
        .await
        .map_err(|err| {
            debug!("Failed to delete schoology request token: {:?}", err);
        })?;

    Ok(())
}
