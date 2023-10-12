use schoology::oauth::get_oauth_request_token;
use serde::Serialize;
use uuid::Uuid;

use crate::{
    database::get_db_client,
    schoology::get_schoology_client,
    utils,
    v1::{RequestData, ResponseError},
    v1_get,
};

#[derive(Serialize)]
struct Response {
    pub id: Uuid,
    pub signature: String,
    pub access_token: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
enum Error {
    SchoologyError,
    DatabaseError,
}

async fn get(_: RequestData<()>) -> Result<Response, ResponseError<Error>> {
    let schoology_client = get_schoology_client();
    let db_client = get_db_client();

    // Get the request token
    let request_token = get_oauth_request_token(&schoology_client)
        .await
        .map_err(|_| ResponseError::ClientError(Error::SchoologyError))?;

    // Create a new entry in the database
    let entry = utils::schoology_request_tokens::create(
        &db_client,
        request_token.access_token.clone(),
        request_token.token_secret.clone(),
        request_token.ttl as usize,
    )
    .await
    .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?;

    // Generate a signature
    let signature =
        utils::schoology_request_tokens::sign(entry.id, request_token.token_secret.clone())
            .map_err(|_| ResponseError::ServerError(Error::DatabaseError))?;

    // Return the response
    // Example link: https://app.schoology.com/oauth/authorize?oauth_callback=example.com&access_token=<access_token>
    Ok(Response {
        id: entry.id,
        signature,
        access_token: request_token.access_token,
        expires_at: entry.expires_at.and_utc(),
    })
}

v1_get!(get_handler, get, NoAuth, Response, Error);
