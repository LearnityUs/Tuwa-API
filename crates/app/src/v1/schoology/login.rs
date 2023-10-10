use actix_web::{post, web};
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use orm::schoology_request_tokens;
use schoology::{
    oauth::{get_oauth_access_token, get_oauth_request_token},
    users::{get_schoology_user, get_user_id},
};
use sea_orm::{ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use sha2::Sha512;
use uuid::Uuid;

use crate::{
    database::get_db_client,
    schoology::get_schoology_client,
    utils::{
        create_default_user, create_schoology_user, create_session,
        get_schoology_user_by_schoology_id, parse_json, update_schoology_user,
    },
    v1::types::{ErrorResponseStatus, ResponseData},
};

#[derive(Deserialize)]
pub struct V1SchoologyLoginServiceRequest {
    pub uuid: Uuid,
    pub signature: String,
    pub login: bool, // The user may need to reauthorize the app but they are already logged in
}

#[derive(Serialize)]
pub struct V1SchoologyLoginServiceResponse {
    pub id: usize,
    pub session_token: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

type Response = ResponseData<V1SchoologyLoginServiceResponse, ()>;

#[post("/login")]
async fn post_login(bytes: web::Bytes, req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    let data = match parse_json::<V1SchoologyLoginServiceRequest>(bytes) {
        Ok(data) => data,
        Err(err) => return err,
    };

    let schoology_client = get_schoology_client();

    let db_client = get_db_client();

    println!("uuid: {:?}", data.uuid);

    // Find the uuid in the database (if not expired)
    let request_token = match schoology_request_tokens::Entity::find_by_id(data.uuid.clone())
        .filter(schoology_request_tokens::Column::ExpiresAt.gt(chrono::Utc::now()))
        .one(db_client)
        .await
    {
        Ok(request_token) => request_token,
        Err(err) => {
            // Error occurred while querying the database
            error!("Failed to query database: {:?}", err);

            let error: Response = ResponseData::error(
                (),
                Some("Failed to query database.".to_string()),
                ErrorResponseStatus::InternalServerError,
            );

            return error.into_response();
        }
    };

    // Check if the request token exists
    let request_token = match request_token {
        Some(request_token) => request_token,
        None => {
            // Request token doesn't exist
            let error: Response = ResponseData::error(
                (),
                Some("Request token doesn't exist.".to_string()),
                ErrorResponseStatus::BadRequest,
            );

            return error.into_response();
        }
    };

    // Hash the uuid with the signature
    let hash = match Hmac::<Sha512>::new_from_slice(request_token.token_secret.as_bytes()) {
        Ok(mut hmac) => {
            hmac.update(data.uuid.as_bytes());
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

    // Check if the signature matches
    if hash != data.signature {
        // Signature doesn't match
        let error: Response = ResponseData::error(
            (),
            Some("Signature doesn't match.".to_string()),
            ErrorResponseStatus::BadRequest,
        );

        return error.into_response();
    }

    // Get the access token
    let access_token = match get_oauth_access_token(
        &schoology_client,
        &request_token.token,
        &request_token.token_secret,
    )
    .await
    {
        Ok(access_token) => access_token,
        Err(err) => match err {
            // 401 Unauthorized means the user didn't authorize the app
            schoology::SchoologyError::Unauthorized => {
                let error: Response = ResponseData::error(
                    (),
                    Some("User didn't authorize the app.".to_string()),
                    ErrorResponseStatus::BadRequest,
                );

                return error.into_response();
            }
            _ => {
                error!("Failed to get access token: {:?}", err);
                let error: Response = ResponseData::error(
                    (),
                    Some("Failed to get access token.".to_string()),
                    ErrorResponseStatus::InternalServerError,
                );

                return error.into_response();
            }
        },
    };

    // Delete the request token from the database
    match request_token.delete(db_client).await {
        Ok(_) => (),
        Err(err) => {
            // Error occurred while deleting the request token
            warn!("Failed to delete request token: {:?}", err);
            // It's not a big deal if the request token isn't deleted
        }
    };

    // Get the user id and user info
    let schoology_id = match get_user_id(
        &schoology_client,
        &access_token.oauth_token,
        &access_token.oauth_token_secret,
    )
    .await
    {
        Ok(user_details) => user_details,
        Err(err) => {
            error!("Failed to get user id: {:?}", err);
            let error: Response = ResponseData::error(
                (),
                Some("Failed to get user id.".to_string()),
                ErrorResponseStatus::InternalServerError,
            );

            return error.into_response();
        }
    };
    // Parse the user id as a usize
    let schoology_id = match schoology_id.user_id.parse::<usize>() {
        Ok(schoology_id) => schoology_id,
        Err(err) => {
            error!("Failed to parse user id: {:?}", err);
            let error: Response = ResponseData::error(
                (),
                Some("Failed to parse user id.".to_string()),
                ErrorResponseStatus::InternalServerError,
            );

            return error.into_response();
        }
    };

    // Get user info
    let user_info = match get_schoology_user(
        &schoology_client,
        &access_token.oauth_token,
        &access_token.oauth_token_secret,
        schoology_id,
    )
    .await
    {
        Ok(user_info) => user_info,
        Err(err) => {
            error!("Failed to get user info: {:?}", err);
            let error: Response = ResponseData::error(
                (),
                Some("Failed to get user info.".to_string()),
                ErrorResponseStatus::InternalServerError,
            );

            return error.into_response();
        }
    };

    // Check if there is a user with the same schoology id
    let if_user = match get_schoology_user_by_schoology_id(db_client, schoology_id).await {
        Ok(if_user) => if_user,
        Err(err) => {
            error!("Failed to query database: {:?}", err);
            let error: Response = ResponseData::error(
                (),
                Some("Failed to query database.".to_string()),
                ErrorResponseStatus::InternalServerError,
            );

            return error.into_response();
        }
    };

    let user = match if_user {
        // Just update the user
        Some(user) => {
            debug!("Updating user with schoology id: {}", schoology_id);

            match update_schoology_user(
                db_client,
                user.user_id as usize,
                Some(user_info.primary_email),
                Some(user_info.picture_url),
                Some(access_token.oauth_token.to_string()),
                Some(access_token.oauth_token_secret.to_string()),
            )
            .await
            {
                Ok(user) => user,
                Err(err) => {
                    error!("Failed to update user: {:?}", err);
                    let error: Response = ResponseData::error(
                        (),
                        Some("Failed to update user.".to_string()),
                        ErrorResponseStatus::InternalServerError,
                    );

                    return error.into_response();
                }
            }
            user
        }
        None => {
            debug!("Creating new user with schoology id: {}", schoology_id);
            // Create a new user
            let user = match create_default_user(db_client).await {
                Ok(user) => user,
                Err(err) => {
                    error!("Failed to create user: {:?}", err);
                    let error: Response = ResponseData::error(
                        (),
                        Some("Failed to create user.".to_string()),
                        ErrorResponseStatus::InternalServerError,
                    );

                    return error.into_response();
                }
            };

            // Create a new schoology user
            match create_schoology_user(
                db_client,
                user.id as usize,
                schoology_id,
                Some(user_info.primary_email),
                Some(user_info.picture_url),
                Some(access_token.oauth_token.to_string()),
                Some(access_token.oauth_token_secret.to_string()),
            )
            .await
            {
                Ok(user) => user,
                Err(err) => {
                    error!("Failed to create schoology user: {:?}", err);
                    let error: Response = ResponseData::error(
                        (),
                        Some("Failed to create schoology user.".to_string()),
                        ErrorResponseStatus::InternalServerError,
                    );

                    return error.into_response();
                }
            }
        }
    };

    let response: Response = ResponseData::success(V1SchoologyLoginServiceResponse {
        id: user.user_id as usize,
        session_token: match data.login {
            true => {
                // User ip
                let ip = req.connection_info();

                // Create a new session
                let session =
                    match create_session(db_client, user.user_id as usize, ip.host().to_string())
                        .await
                    {
                        Ok(session) => session,
                        Err(err) => {
                            error!("Failed to create session: {:?}", err);
                            let error: Response = ResponseData::error(
                                (),
                                Some("Failed to create session.".to_string()),
                                ErrorResponseStatus::InternalServerError,
                            );

                            return error.into_response();
                        }
                    };

                Some(session.1)
            }
            false => None,
        },
        expires_at: chrono::Utc::now() + chrono::Duration::days(1),
    });

    response.into_response()
}
