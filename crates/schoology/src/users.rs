use reqwest::StatusCode;
use serde::Deserialize;

use crate::{SchoologyClient, SchoologyError, SchoologyRequest, BASE_URL};

#[derive(Debug)]
pub struct UsersUserIdResponse {
    pub user_id: String,
}

/// This get's the user id because schoology poorly designed their apo.
/// The `/users/me` endpoint is just a redirect to `/users/{user_id}`.
/// This causes the `Duplicate timestamp/nonce combination, possible replay attack.` error.
/// This function just ignores the redirect and gets the user id from the redirect url.
/// This is a workaround for the schoology api.
pub async fn get_user_id(
    client: &SchoologyClient,
    oauth_token: &str,
    oauth_token_secret: &str,
) -> Result<UsersUserIdResponse, SchoologyError> {
    debug!("Getting user id");

    // Make the request (ignore the redirect)
    let response = client
        .get(
            "/v1/users/me",
            SchoologyRequest::new()
                .with_oauth_tokens(oauth_token, oauth_token_secret)
                .redirects(false),
        )
        .await;

    println!("response: {}, {}", oauth_token, oauth_token_secret);

    let response = match response {
        Ok(response) => match &response.status() {
            &StatusCode::SEE_OTHER => response,
            &StatusCode::BAD_REQUEST => return Err(SchoologyError::BadRequest),
            &StatusCode::UNAUTHORIZED => return Err(SchoologyError::Unauthorized),
            &StatusCode::FORBIDDEN => return Err(SchoologyError::Forbidden),
            &StatusCode::NOT_FOUND => return Err(SchoologyError::NotFound),
            &StatusCode::INTERNAL_SERVER_ERROR => return Err(SchoologyError::InternalServerError),
            _ => {
                debug!("Unknown status code: {}", response.status());
                return Err(SchoologyError::Other(format!(
                    "Unknown status code: {}",
                    response.status()
                )));
            }
        },
        Err(err) => return Err(SchoologyError::RequestError(err)),
    };

    // Get the response headers
    let response = response.headers();

    // Get the redirect url
    let location = match response.get("location") {
        Some(location) => match location.to_str() {
            Ok(location) => location,
            Err(err) => {
                warn!("Failed to get location header: {:?}", err);
                return Err(SchoologyError::Other(format!(
                    "Failed to get location header: {:?}",
                    err
                )));
            }
        },
        None => {
            debug!("No location header");
            return Err(SchoologyError::Other("No location header".to_string()));
        }
    };

    // Get the user id from the redirect url (parse the path)
    let url = match BASE_URL.join(location) {
        Ok(url) => url,
        Err(err) => {
            warn!("Failed to parse redirect url: {:?}", err);
            return Err(SchoologyError::Other(format!(
                "Failed to parse redirect url: {:?}",
                err
            )));
        }
    };

    // Get the user id from the path (the last segment)
    match url.path_segments() {
        Some(mut segments) => match segments.next_back() {
            Some(user_id) => Ok(UsersUserIdResponse {
                user_id: user_id.to_string(),
            }),
            None => {
                debug!("No user id in path");
                Err(SchoologyError::Other("No user id in path".to_string()))
            }
        },
        None => {
            debug!("No path segments");
            Err(SchoologyError::Other("No path segments".to_string()))
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct SchoologyUser {
    pub id: usize,
    pub school_id: usize,
    pub name_first: String,
    pub name_last: String,
    pub primary_email: String,
    pub picture_url: String,
}

/// Gets a schoology user
pub async fn get_schoology_user(
    client: &SchoologyClient,
    oauth_token: &str,
    oauth_token_secret: &str,
    user_id: usize,
) -> Result<SchoologyUser, SchoologyError> {
    debug!("Getting schoology user {}", user_id);

    // Make the request
    let response = client
        .get(
            &format!("/v1/users/{}", urlencoding::encode(&user_id.to_string())),
            SchoologyRequest::new()
                .with_oauth_tokens(oauth_token, oauth_token_secret)
                .redirects(false),
        )
        .await;

    let response = match response {
        Ok(response) => match &response.status() {
            &StatusCode::OK => response.text().await,
            &StatusCode::BAD_REQUEST => return Err(SchoologyError::BadRequest),
            &StatusCode::UNAUTHORIZED => return Err(SchoologyError::Unauthorized),
            &StatusCode::FORBIDDEN => return Err(SchoologyError::Forbidden),
            &StatusCode::NOT_FOUND => return Err(SchoologyError::NotFound),
            &StatusCode::INTERNAL_SERVER_ERROR => return Err(SchoologyError::InternalServerError),
            _ => {
                debug!("Unknown status code: {}", response.status());
                return Err(SchoologyError::Other(format!(
                    "Unknown status code: {}",
                    response.status()
                )));
            }
        },
        Err(err) => return Err(SchoologyError::RequestError(err)),
    };

    let response = match response {
        Ok(response) => response,
        Err(err) => return Err(SchoologyError::RequestError(err)),
    };

    // Parse the response
    let response: SchoologyUser = match serde_json::from_str(&response) {
        Ok(response) => response,
        Err(err) => return Err(SchoologyError::SerdeJSONError(err)),
    };

    Ok(response)
}
