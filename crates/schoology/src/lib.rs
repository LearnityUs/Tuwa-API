use once_cell::sync::Lazy;
use url::Url;

static BASE_URL: Lazy<Url> = Lazy::new(|| Url::parse("https://api.schoology.com/v1/").unwrap());

#[macro_use]
extern crate log;

pub mod oauth;

pub struct SchoologyClient {
    pub client: reqwest::Client,
    pub consumer_key: String,
    pub consumer_secret: String
}

/// A request to the Schoology API
pub struct SchoologyRequest {
    /// Query parameters for the request
    pub query: Option<Vec<(String, String)>>,
    /// For JSON requests (ignored in GET requests)
    pub body: Option<String>,
    /// For url encoded `application/x-www-form-urlencoded` requests
    pub oauth_body: Option<Vec<(String, String)>>,
    /// User token
    pub oauth_token: Option<String>,
    /// User token secret
    pub oauth_token_secret: Option<String>,
}

impl SchoologyRequest {
    /// Creates a new SchoologyRequest
    pub fn new(
        query: Option<Vec<(String, String)>>,
        body: Option<String>,
        oauth_body: Option<Vec<(String, String)>>,
        oauth_token: Option<String>,
        oauth_token_secret: Option<String>,
    ) -> Self {
        Self {
            query,
            body,
            oauth_body,
            oauth_token,
            oauth_token_secret,
        }
    }

    /// Empty request
    pub fn empty() -> Self {
        Self::new(None, None, None, None, None)
    }

    /// Empty with oauth token and secret
    pub fn empty_with_oauth_token(oauth_token: String, oauth_token_secret: String) -> Self {
        Self::new(None, None, None, Some(oauth_token), Some(oauth_token_secret))
    }
}

impl SchoologyClient {
    /// Creates a new SchoologyClient
    pub fn new(consumer_key: String, consumer_secret: String) -> Self {
        debug!("Creating new SchoologyClient");
        Self {
            client: reqwest::Client::new(),
            consumer_key,
            consumer_secret
        }
    }

    /// Creates a get request to the Schoology API
    /// Note: Do NOT allow arbitrary paths to be passed in. This allows for path traversal attacks.
    pub async fn get(&self, path: &str, request: SchoologyRequest) -> Result<reqwest::Response, reqwest::Error> {
        debug!("Schoology API v1 GET request to {}", path);
        
        // Create the request URL
        let mut url = BASE_URL.join(path).unwrap();

        // Add the query parameters
        let url = if let Some(query) = request.query {
            url.query_pairs_mut().extend_pairs(query);
            url
        } else {
            url
        };

        // Oauth 1.0a header
        let oauth_header = oauth::OAuth1AHeader::new(
            "GET".to_string(),
            url.to_string(),
            request.oauth_token,
            request.oauth_token_secret
        );


        // Generate the signature
        let signature = oauth_header.get_header(
            "GET",
            &url,
            None,
            &self.consumer_key,
            &self.consumer_secret
        );

        debug!("Generated signature: {}", signature);

        let req = self.client.get(url)
            .header("Accept", "application/json")
            .header("Authorization", signature);

        req.send().await
    }
}