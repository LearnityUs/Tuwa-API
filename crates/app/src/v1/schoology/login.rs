//! [Docs](/docs/api/v1/schoology/login)

use schoology::{oauth, users, SchoologyTokenPair};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    database::get_db_client,
    schoology::get_schoology_client,
    utils,
    v1::{
        types::{ErrorResponseStatus, ResponseData},
        RequestData, ResponseError,
    },
    v1_post,
};

#[derive(Deserialize)]
pub struct Request {
    pub uuid: Uuid,
    pub signature: String,
    pub login: bool, // The user may need to reauthorize the app but they are already logged in
}

#[derive(Serialize)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
enum Error {
    SchoologyError,
    DatabaseError,
    InvalidFlowId,
    InvalidSignature,
    ApplicationNotAuthorized,
}

async fn post(req: RequestData<Request>) -> Result<Response, ResponseError<Error>> {
    let schoology_client = get_schoology_client();

    let db_client = get_db_client();

    // Find the uuid in the database (if not expired)
    let request_token = utils::schoology_request_tokens::get(&db_client, req.data.uuid)
        .await
        .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?
        .ok_or(ResponseError::ClientError(Error::InvalidFlowId))?;

    // Check past the expiration date
    if request_token.expires_at < chrono::Utc::now().naive_utc() {
        // Request token expired
        debug!("Request token expired");
        return Err(ResponseError::ClientError(Error::InvalidFlowId));
    }

    // Verify that the signature matches
    let signature =
        utils::schoology_request_tokens::sign(request_token.id, request_token.token_secret.clone())
            .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?;

    // Check if the signature matches
    if signature != req.data.signature {
        // Signature doesn't match
        debug!("Signature doesn't match");
        return Err(ResponseError::ClientError(Error::InvalidSignature));
    }

    // Construct the token
    let token = SchoologyTokenPair {
        access_token: request_token.access_token.clone(),
        token_secret: request_token.token_secret.clone(),
    };

    // Get the access token
    let token = oauth::get_oauth_access_token(&schoology_client, &token)
        .await
        .map_err(|_| ResponseError::ClientError(Error::ApplicationNotAuthorized))?;

    // Delete the request token from the database
    utils::schoology_request_tokens::delete(&db_client, request_token.id)
        .await
        .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?;

    // Get the user id and user info
    let schoology_id = users::get_user_id(&schoology_client, &token)
        .await
        .map_err(|_| ResponseError::ClientError(Error::SchoologyError))?;

    // Get user info
    let user_info = users::get_schoology_user(schoology_client, &token, schoology_id)
        .await
        .map_err(|_| ResponseError::ClientError(Error::SchoologyError))?;

    // Check if there is a user with the same schoology id
    let link = utils::schoology_link::get(&db_client, schoology_id as i32)
        .await
        .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?;

    let link = match link {
        // Just update the user
        Some(user) => {
            debug!("Updating user with schoology id: {}", schoology_id);

            // Update the user
            utils::schoology_link::update(
                &db_client,
                user.user_id as i32,
                Some(user_info.primary_email),
                Some(user_info.picture_url),
                Some(token.access_token.to_string()),
                Some(token.token_secret.to_string()),
                Some(token.access_token.to_string()),
                Some(token.token_secret.to_string()),
            )
            .await
        }
        None => {
            debug!("Creating new user with schoology id: {}", schoology_id);
            // Create a new user
            let user = utils::users::create(db_client)
                .await
                .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?;

            // Create a new link
            utils::schoology_link::create(
                &db_client,
                user.id,
                schoology_id as i32,
                Some(user_info.primary_email),
                Some(user_info.picture_url),
                Some(token.access_token.to_string()),
                Some(token.token_secret.to_string()),
                Some(token.access_token.to_string()),
                Some(token.token_secret.to_string()),
            )
            .await
        }
    }
    .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?;

    let session = match req.data.login {
        true => {
            // User ip
            let ip = req.http_request.connection_info();

            // Create a new session
            let session =
                utils::sessions::create(&db_client, link.user_id as i32, ip.host().to_string())
                    .await
                    .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?;

            Some(session)
        }
        false => None,
    };

    Ok(Response {
        session_token: match session {
            Some(ref session) => Some(
                utils::sessions::encode(utils::sessions::AccessToken::user(
                    session.id,
                    session.token.clone(),
                ))
                .await
                .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?,
            ),
            None => None,
        },
        session_expires_at: match session {
            Some(ref session) => Some(session.expires_at.and_utc()),
            None => None,
        },
    })
}

v1_post!(post_handler, post, NoAuth, Request, Response, Error);
