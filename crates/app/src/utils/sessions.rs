use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use orm::sessions;
use ring::rand::{SecureRandom, SystemRandom};
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The access token struct
#[derive(Serialize, Deserialize)]
pub struct AccessTokenUser {
    id: String,
    signature: String,
}

/// The token enum
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AccessToken {
    User(AccessTokenUser),
}

impl AccessToken {
    /// Creates a new user access token
    pub fn user(id: Uuid, signature: String) -> AccessToken {
        AccessToken::User(AccessTokenUser {
            id: id.to_string(),
            signature,
        })
    }
}

/// Decodes a session from it's string representation
pub async fn decode(string: &str) -> Result<AccessToken, ()> {
    // Decode the base64
    let decoded = match STANDARD_NO_PAD.decode(string) {
        Ok(decoded) => decoded,
        Err(err) => {
            debug!("Failed to decode session: {:?}", err);
            return Err(());
        }
    };

    // Parse the session
    let access_token: AccessToken = match serde_json::from_slice(&decoded) {
        Ok(session) => session,
        Err(err) => {
            debug!("Failed to parse session: {:?}", err);
            return Err(());
        }
    };

    Ok(access_token)
}

/// Encodes a session to it's string representation
pub async fn encode(access_token: AccessToken) -> Result<String, ()> {
    // Encode the session
    let access_token = serde_json::to_string(&access_token).map_err(|err| {
        debug!("Failed to encode session: {:?}", err);
    })?;
    // Encode the base64
    Ok(STANDARD_NO_PAD.encode(access_token.as_bytes()))
}

/// Get's a session from the database
pub async fn get(
    db_client: &DatabaseConnection,
    session_id: Uuid,
) -> Result<Option<sessions::Model>, ()> {
    let session = sessions::Entity::find_by_id(session_id)
        .one(db_client)
        .await
        .map_err(|err| {
            debug!("Failed to get session: {:?}", err);
        })?;

    Ok(session)
}

/// Creates a session for a user
pub async fn create(
    db_client: &DatabaseConnection,
    user_id: i32,
    ip: String,
) -> Result<sessions::Model, ()> {
    // Generate a new secret
    let engine = SystemRandom::new();
    let mut secret = [0u8; 32];
    engine.fill(&mut secret).map_err(|err| {
        error!("Failed to generate secret: {:?}", err);
    })?;

    // Base64 encode the secret
    let secret = STANDARD_NO_PAD.encode(&secret);

    let session = sessions::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        user_id: ActiveValue::Set(user_id),
        token: ActiveValue::Set(secret),
        initial_ip: ActiveValue::Set(ip),
        expires_at: ActiveValue::Set((chrono::Utc::now() + chrono::Duration::days(30)).naive_utc()),
    };

    let session = session.insert(db_client).await.map_err(|err| {
        debug!("Failed to create session: {:?}", err);
    })?;

    Ok(session)
}

/// Deletes a session from the database
pub async fn delete(db_client: &DatabaseConnection, session_id: Uuid) -> Result<(), ()> {
    sessions::Entity::delete_by_id(session_id)
        .exec(db_client)
        .await
        .map_err(|err| {
            debug!("Failed to get session: {:?}", err);
        })?;

    Ok(())
}

/// Verifies a session
pub async fn verify(
    db_client: &DatabaseConnection,
    token: &str,
) -> Result<Option<sessions::Model>, ()> {
    // Decode the session
    let access_token = decode(token).await?;

    // Get the session
    match access_token {
        AccessToken::User(user) => {
            // Get the session
            let session = get(
                db_client,
                Uuid::parse_str(&user.id).map_err(|err| {
                    debug!("Failed to parse session id: {:?}", err);
                })?,
            )
            .await?
            .ok_or(())?;

            // Check token
            if session.token != user.signature {
                debug!("Invalid token");
                return Ok(None);
            }

            Ok(Some(session))
        }
    }
}
