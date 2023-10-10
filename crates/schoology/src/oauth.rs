use reqwest::StatusCode;
use serde::Deserialize;
use serde_urlencoded;

use crate::{SchoologyClient, SchoologyError, SchoologyRequest};

#[derive(Deserialize, Debug)]
pub struct OauthRequestTokenResponse {
    /// The oauth token
    pub oauth_token: String,
    /// The oauth token secret
    pub oauth_token_secret: String,
    /// The oauth token time to live
    pub xoauth_token_ttl: usize,
}

/// Sends a request to the Schoology API to get an oauth request token
pub async fn get_oauth_request_token(
    client: &SchoologyClient,
) -> Result<OauthRequestTokenResponse, SchoologyError> {
    debug!("Getting oauth request token");
    let response = client
        .get("/v1/oauth/request_token", SchoologyRequest::new())
        .await;

    let response = match response {
        Ok(response) => match response.status() {
            StatusCode::OK => response.text().await,
            StatusCode::BAD_REQUEST => return Err(SchoologyError::BadRequest),
            StatusCode::UNAUTHORIZED => return Err(SchoologyError::Unauthorized),
            StatusCode::FORBIDDEN => return Err(SchoologyError::Forbidden),
            StatusCode::NOT_FOUND => return Err(SchoologyError::NotFound),
            StatusCode::INTERNAL_SERVER_ERROR => return Err(SchoologyError::InternalServerError),
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
    let response: OauthRequestTokenResponse = match serde_urlencoded::from_str(&response) {
        Ok(response) => response,
        Err(err) => return Err(SchoologyError::SerdeURLError(err)),
    };

    Ok(response)
}

#[derive(Deserialize, Debug)]
pub struct OauthAccessTokenResponse {
    /// The oauth token
    pub oauth_token: String,
    /// The oauth token secret
    pub oauth_token_secret: String,
}

/// Sends a request to the Schoology API to get an oauth access token
/// This uses the request_token's oauth_token and oauth_token_secret
pub async fn get_oauth_access_token(
    client: &SchoologyClient,
    oauth_token: &str,
    token_secret: &str,
) -> Result<OauthAccessTokenResponse, SchoologyError> {
    debug!("Getting oauth access token");
    let response = client
        .get(
            "/v1/oauth/access_token",
            SchoologyRequest::new().with_oauth_tokens(oauth_token, token_secret),
        )
        .await;

    let response = match response {
        Ok(response) => match response.status() {
            StatusCode::OK => response.text().await,
            StatusCode::BAD_REQUEST => return Err(SchoologyError::BadRequest),
            StatusCode::UNAUTHORIZED => return Err(SchoologyError::Unauthorized),
            StatusCode::FORBIDDEN => return Err(SchoologyError::Forbidden),
            StatusCode::NOT_FOUND => return Err(SchoologyError::NotFound),
            StatusCode::INTERNAL_SERVER_ERROR => return Err(SchoologyError::InternalServerError),
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
    let response: OauthAccessTokenResponse = match serde_urlencoded::from_str(&response) {
        Ok(response) => response,
        Err(err) => return Err(SchoologyError::SerdeURLError(err)),
    };

    Ok(response)
}
