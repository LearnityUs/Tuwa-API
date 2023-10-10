use actix_web::web;
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use orm::{schoology_link, sessions, users};
use ring::rand::{self, SecureRandom};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use uuid::Uuid;

use crate::v1::types::{ErrorResponseStatus, ResponseData};

/// Parses or returns that the json is invalid
pub fn parse_json<T: DeserializeOwned>(bytes: web::Bytes) -> Result<T, actix_web::HttpResponse> {
    // Convert the bytes to a slice
    let bytes = bytes.as_ref();

    match serde_json::from_slice::<T>(bytes) {
        Ok(v) => Ok(v),
        Err(e) => {
            debug!("Failed to parse json: {}", e);

            // Return a bad request
            let error: ResponseData<(), ()> = ResponseData::error(
                (),
                Some("Failed to parse json.".to_string()),
                ErrorResponseStatus::BadRequest,
            );

            Err(error.into_response())
        }
    }
}

/// Creates a user
pub async fn create_default_user(db_client: &DatabaseConnection) -> Result<users::Model, DbErr> {
    let user = users::ActiveModel {
        id: ActiveValue::NotSet,
        is_admin: ActiveValue::set(false),
        is_root: ActiveValue::set(false),
        created_at: ActiveValue::set(chrono::Utc::now().naive_utc()),
    };

    let user = user.insert(db_client).await;

    match user {
        Ok(user) => Ok(user),
        Err(err) => {
            error!("Failed to create default user: {:?}", err);
            Err(err)
        }
    }
}

/// Creates a schoology user
pub async fn create_schoology_user(
    db_client: &DatabaseConnection,
    user_id: usize,
    schoology_id: usize,
    email: Option<String>,
    picture_url: Option<String>,
    access_token: Option<String>,
    access_token_secret: Option<String>,
) -> Result<schoology_link::Model, DbErr> {
    let schoology_user = schoology_link::ActiveModel {
        user_id: ActiveValue::Set(user_id as i32),
        schoology_id: ActiveValue::set(schoology_id as i32),
        email: ActiveValue::set(email),
        picture_url: ActiveValue::set(picture_url),
        access_token: ActiveValue::set(access_token),
        token_secret: ActiveValue::set(access_token_secret),
    };

    let schoology_user = schoology_user.insert(db_client).await;

    match schoology_user {
        Ok(schoology_user) => Ok(schoology_user),
        Err(err) => {
            warn!("Failed to create schoology user: {:?}", err);
            Err(err)
        }
    }
}

/// Gets a schoology user by the user id
pub async fn get_schoology_user_by_schoology_id(
    db_client: &DatabaseConnection,
    user_id: usize,
) -> Result<Option<schoology_link::Model>, DbErr> {
    let schoology_user = schoology_link::Entity::find()
        .filter(schoology_link::Column::SchoologyId.eq(user_id as i32))
        .one(db_client)
        .await;

    match schoology_user {
        Ok(schoology_user) => Ok(schoology_user),
        Err(err) => {
            warn!("Failed to get schoology user: {:?}", err);
            Err(err)
        }
    }
}

/// Update a schoology user (new email, picture url, access token, access token secret)
pub async fn update_schoology_user(
    db_client: &DatabaseConnection,
    user_id: usize,
    email: Option<String>,
    picture_url: Option<String>,
    access_token: Option<String>,
    access_token_secret: Option<String>,
) -> Result<(), DbErr> {
    let schoology_user = schoology_link::ActiveModel {
        user_id: ActiveValue::Set(user_id as i32),
        email: ActiveValue::set(email),
        picture_url: ActiveValue::set(picture_url),
        access_token: ActiveValue::set(access_token),
        token_secret: ActiveValue::set(access_token_secret),
        ..Default::default()
    };

    let schoology_user = schoology_user.update(db_client).await;

    match schoology_user {
        Ok(_) => Ok(()),
        Err(err) => {
            warn!("Failed to update schoology user: {:?}", err);
            Err(err)
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AccessTokenUser {
    id: String,
    signature: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum AccessToken {
    User(AccessTokenUser),
}

/// Creates a session for a user
/// Returns the session id and the session token
/// return.0 - The session id
/// return.1 - The session token send to user
pub async fn create_session(
    db_client: &DatabaseConnection,
    user_id: usize,
    initial_ip: String,
) -> Result<(Uuid, String), DbErr> {
    // Secure generate a token
    let mut buf = [0u8; 32];
    let sys_rand = rand::SystemRandom::new();

    sys_rand.fill(&mut buf).unwrap();

    let token = STANDARD_NO_PAD.encode(&buf);

    // 30 days from now
    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);

    let session = sessions::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        user_id: ActiveValue::Set(user_id as i32),
        token: ActiveValue::set(token),
        initial_ip: ActiveValue::set(initial_ip),
        expires_at: ActiveValue::set(expires_at.naive_utc()),
    };

    let session = session.insert(db_client).await;

    match session {
        Ok(session) => {
            // Encode the session id and signature
            let access_token = AccessToken::User(AccessTokenUser {
                id: session.id.to_string(),
                signature: session.token,
            });

            let access_token = serde_json::to_string(&access_token).unwrap();

            // Base64 encode the access token
            let access_token = STANDARD_NO_PAD.encode(access_token.as_bytes());

            Ok((session.id, access_token))
        }
        Err(err) => {
            warn!("Failed to create session: {:?}", err);
            Err(err)
        }
    }
}
