use reqwest::StatusCode;
use serde::Deserialize;

use crate::{SchoologyClient, SchoologyRequest, SchoologyTokenPair, BASE_URL};

pub enum GetUserIdError {
    /// Not found
    NotFound,
    /// Unauthorized
    Unauthorized,
    /// Other
    Other,
}

/// This get's the user id because schoology poorly designed their apo.
/// The `/users/me` endpoint is just a redirect to `/users/{user_id}`.
/// This causes the `Duplicate timestamp/nonce combination, possible replay attack.` error.
/// This function just ignores the redirect and gets the user id from the redirect url.
/// This is a workaround for the schoology api.
pub async fn get_user_id(
    client: &SchoologyClient,
    token: &SchoologyTokenPair,
) -> Result<usize, GetUserIdError> {
    debug!("Getting user id");

    // Make the request (ignore the redirect)
    let response = client
        .get(
            "/v1/users/me",
            SchoologyRequest::new()
                .with_access_tokens(token)
                .redirects(false),
        )
        .await;

    let response = match response {
        Ok(response) => match &response.status() {
            &StatusCode::SEE_OTHER => Ok(response),
            &StatusCode::NOT_FOUND => {
                debug!("User not found");
                Err(GetUserIdError::NotFound)
            }
            &StatusCode::UNAUTHORIZED => {
                debug!("Unauthorized: may be because the session token is expired");
                Err(GetUserIdError::Unauthorized)
            }
            _ => {
                debug!("Unknown status code: {}", response.status());
                return Err(GetUserIdError::Other);
            }
        },
        Err(err) => {
            warn!("Failed to get user id: {:?}", err);
            return Err(GetUserIdError::Other);
        }
    }?;

    // Get the response headers
    let response = response.headers();

    // Get the redirect url
    let location = response.get("location").ok_or_else(|| {
        debug!("No location header");
        GetUserIdError::Other
    })?;

    // Get the redirect url as a string
    let location = location.to_str().map_err(|err| {
        warn!("Failed to convert location header to string: {:?}", err);
        GetUserIdError::Other
    })?;

    // Get the user id from the redirect url (parse the path)
    let url = BASE_URL.join(location).map_err(|err| {
        warn!("Failed to parse redirect url: {:?}", err);
        GetUserIdError::Other
    })?;

    // Get the user id from the path (the last segment)
    match url.path_segments() {
        Some(mut segments) => match segments.next_back() {
            Some(user_id) => Ok(match user_id.parse() {
                Ok(user_id) => user_id,
                Err(err) => {
                    warn!("Failed to parse user id: {:?}", err);
                    return Err(GetUserIdError::Other);
                }
            }),
            None => {
                debug!("No user id in path");
                Err(GetUserIdError::Other)
            }
        },
        None => {
            debug!("No path segments");
            Err(GetUserIdError::Other)
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

pub enum GetSchoologyUserError {
    /// Unauthorized
    Unauthorized,
    /// Other
    Other,
}

/// Gets a schoology user
pub async fn get_schoology_user(
    client: &SchoologyClient,
    token: &SchoologyTokenPair,
    user_id: usize,
) -> Result<SchoologyUser, GetSchoologyUserError> {
    debug!("Getting schoology user {}", user_id);

    // Make the request
    let response = client
        .get(
            &format!("/v1/users/{}", urlencoding::encode(&user_id.to_string())),
            SchoologyRequest::new()
                .with_access_tokens(token)
                .redirects(false),
        )
        .await;

    let response = match response {
        Ok(response) => match &response.status() {
            &StatusCode::OK => response.text().await.map_err(|err| {
                warn!("Failed to get schoology user: {:?}", err);
                GetSchoologyUserError::Other
            }),
            &StatusCode::UNAUTHORIZED => {
                debug!("Unauthorized: may be because the session token is expired");
                Err(GetSchoologyUserError::Unauthorized)
            }
            _ => {
                debug!("Unknown status code: {}", response.status());
                Err(GetSchoologyUserError::Other)
            }
        },
        Err(err) => {
            warn!("Failed to get schoology user: {:?}", err);
            Err(GetSchoologyUserError::Other)
        }
    }?;

    // Parse the response
    let response: SchoologyUser = serde_json::from_str(&response).map_err(|err| {
        warn!("Failed to parse schoology user: {:?}", err);
        GetSchoologyUserError::Other
    })?;

    Ok(response)
}
