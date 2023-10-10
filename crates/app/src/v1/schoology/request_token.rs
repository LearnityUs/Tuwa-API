use actix_web::get;
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use orm::schoology_request_tokens;
use schoology::oauth::get_oauth_request_token;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serde::Serialize;
use sha2::Sha512;
use uuid::Uuid;

use crate::{
    database::get_db_client,
    schoology::get_schoology_client,
    v1::types::{ErrorResponseStatus, ResponseData},
};

#[derive(Serialize)]
pub struct V1SchoologyRequestTokenServiceResponse {
    pub uuid: Uuid,
    pub signature: String,
    pub oauth_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

type Response = ResponseData<V1SchoologyRequestTokenServiceResponse, ()>;

#[get("/request_token")]
async fn get_request_token() -> actix_web::HttpResponse {
    let schoology_client = get_schoology_client();
    let db_client = get_db_client();

    // Get the request token
    let request_token = match get_oauth_request_token(&schoology_client).await {
        Ok(request_token) => request_token,
        Err(err) => {
            error!("Failed to get request token: {:?}", err);
            let error: Response = ResponseData::error(
                (),
                Some("Failed to get request token.".to_string()),
                ErrorResponseStatus::InternalServerError,
            );

            return error.into_response();
        }
    };

    // Generate a uuid
    let uuid = uuid::Uuid::new_v4();

    // Hash the uuid with the signature
    let hash = match Hmac::<Sha512>::new_from_slice(request_token.oauth_token_secret.as_bytes()) {
        Ok(mut hmac) => {
            hmac.update(uuid.as_bytes());
            hmac.finalize()
        }
        Err(err) => {
            error!("Failed to hash request token: {:?}", err);
            let error: Response = ResponseData::error(
                (),
                Some("Failed to hash request token.".to_string()),
                ErrorResponseStatus::InternalServerError,
            );

            return error.into_response();
        }
    };
    // Base64 encode the hash
    let hash = STANDARD_NO_PAD.encode(&hash.into_bytes());

    // Convert UTC to chrono::DateTime<chrono::Utc>
    let expires_at =
        chrono::Utc::now() + chrono::Duration::seconds(request_token.xoauth_token_ttl as i64);

    // Insert the request token into the database
    let db_entry = schoology_request_tokens::ActiveModel {
        id: ActiveValue::Set(uuid),
        token: ActiveValue::Set(request_token.oauth_token.clone()),
        token_secret: ActiveValue::Set(request_token.oauth_token_secret.clone()),
        expires_at: ActiveValue::Set(expires_at.naive_utc()),
    };

    // Insert the request token into the database
    match db_entry.insert(db_client).await {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to insert request token into database: {:?}", err);
            let error: Response = ResponseData::error(
                (),
                Some("Failed to insert request token into database.".to_string()),
                ErrorResponseStatus::InternalServerError,
            );

            return error.into_response();
        }
    }

    // Return the response
    // Example link: https://app.schoology.com/oauth/authorize?oauth_callback=example.com&oauth_token=<oauth_token>
    let response: Response = ResponseData::success(V1SchoologyRequestTokenServiceResponse {
        uuid,
        signature: hash,
        oauth_token: request_token.oauth_token,
        expires_at,
    });

    response.into_response()
}
