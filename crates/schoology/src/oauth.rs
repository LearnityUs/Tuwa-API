use reqwest::StatusCode;
use serde::{de, Deserialize};
use serde_urlencoded;

use crate::{SchoologyClient, SchoologyRequest, SchoologyTokenPair};

#[derive(Deserialize, Debug)]
pub struct OauthRequestTokenResponse {
    /// The oauth token
    pub oauth_token: String,
    /// The oauth token secret
    pub oauth_token_secret: String,
    /// The oauth token time to live
    pub xoauth_token_ttl: usize,
}

pub struct OauthRequestToken {
    /// The oauth token
    pub access_token: String,
    /// The oauth token secret
    pub token_secret: String,
    /// The oauth token time to live
    pub ttl: usize,
}

pub enum OauthRequestTokenError {
    /// Application not authorized
    Unauthorized,
    /// Other error
    Other,
}

/// Sends a request to the Schoology API to get an oauth request token
pub async fn get_oauth_request_token(
    client: &SchoologyClient,
) -> Result<OauthRequestToken, OauthRequestTokenError> {
    debug!("Getting oauth request token");
    let response = client
        .get("/v1/oauth/request_token", SchoologyRequest::new())
        .await;

    let data = match response {
        Ok(response) => match response.status() {
            StatusCode::OK => response.text().await.map_err(|err| {
                warn!("Failed to get oauth request token: {:?}", err);
                OauthRequestTokenError::Other
            }),
            StatusCode::UNAUTHORIZED => {
                warn!("Application not authorized");
                Err(OauthRequestTokenError::Unauthorized)
            }
            _ => {
                debug!("Unknown status code...");
                Err(OauthRequestTokenError::Other)
            }
        },
        Err(err) => {
            warn!("Failed to get oauth request token: {:?}", err);
            Err(OauthRequestTokenError::Other)
        }
    }?;

    // Parse the response
    let response: OauthRequestTokenResponse = serde_urlencoded::from_str(&data).map_err(|err| {
        warn!("Failed to parse oauth request token response: {:?}", err);
        OauthRequestTokenError::Other
    })?;

    Ok(OauthRequestToken {
        access_token: response.oauth_token,
        token_secret: response.oauth_token_secret,
        ttl: response.xoauth_token_ttl,
    })
}

#[derive(Deserialize, Debug)]
pub struct OauthAccessTokenResponse {
    /// The oauth token
    pub oauth_token: String,
    /// The oauth token secret
    pub oauth_token_secret: String,
}

pub enum AccessTokenError {
    /// Expired / non-existent request token or application not authorized
    Unauthorized,
    /// Other error
    Other,
}

/// Sends a request to the Schoology API to get an oauth access token
/// This uses the request_token's access_token and access_token_secret
pub async fn get_oauth_access_token(
    client: &SchoologyClient,
    tokens: &SchoologyTokenPair,
) -> Result<SchoologyTokenPair, AccessTokenError> {
    debug!("Getting oauth access token");
    let response = client
        .get(
            "/v1/oauth/access_token",
            SchoologyRequest::new().with_access_tokens(tokens),
        )
        .await;

    let data = match response {
        Ok(response) => match response.status() {
            StatusCode::OK => response.text().await.map_err(|err| {
                warn!("Failed to get oauth access token: {:?}", err);
                AccessTokenError::Other
            }),
            StatusCode::UNAUTHORIZED => {
                debug!("Token is invalid / expired");
                Err(AccessTokenError::Unauthorized)
            }
            _ => {
                debug!("Unknown status code: {}", response.status());
                Err(AccessTokenError::Other)
            }
        },

        Err(err) => {
            warn!("Failed to get oauth access token: {:?}", err);
            Err(AccessTokenError::Other)
        }
    }?;

    // Parse the response
    let response: OauthAccessTokenResponse = serde_urlencoded::from_str(&data).map_err(|err| {
        warn!("Failed to parse oauth access token response: {:?}", err);
        AccessTokenError::Other
    })?;

    Ok(SchoologyTokenPair {
        access_token: response.oauth_token,
        token_secret: response.oauth_token_secret,
    })
}
