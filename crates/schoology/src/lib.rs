use once_cell::sync::Lazy;
use url::Url;

pub static BASE_URL: Lazy<Url> = Lazy::new(|| Url::parse("https://api.schoology.com/v1/").unwrap());

#[macro_use]
extern crate log;

pub mod oauth;
pub mod proto;
pub mod users;

pub struct SchoologyClient {
    pub consumer_key: String,
    pub consumer_secret: String,
}

/// A token pair representing a user's oauth token and token secret
pub struct SchoologyTokenPair {
    pub access_token: String,
    pub token_secret: String,
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
    pub access_token: Option<String>,
    /// User token secret
    pub token_secret: Option<String>,

    /// Whether or not to follow redirects
    pub redirects: bool,
}

impl SchoologyRequest {
    /// Create a new blank SchoologyRequest
    pub fn new() -> Self {
        Self {
            query: None,
            body: None,
            oauth_body: None,
            access_token: None,
            token_secret: None,
            redirects: true,
        }
    }

    /// Add a query parameter to the request
    pub fn with_query_param(mut self, key: String, value: String) -> Self {
        if let Some(query) = &mut self.query {
            query.push((key, value));
        } else {
            self.query = Some(vec![(key, value)]);
        }

        self
    }

    /// Add a body to the request
    pub fn with_body(mut self, body: String) -> Self {
        self.body = Some(body);
        self
    }

    /// Add a form body to the request
    pub fn with_form_body(mut self, body: Vec<(String, String)>) -> Self {
        self.oauth_body = Some(body);
        self
    }

    /// Add an oauth token to the request
    pub fn with_access_tokens(mut self, token: &SchoologyTokenPair) -> Self {
        self.access_token = Some(token.access_token.clone());
        self.token_secret = Some(token.token_secret.clone());
        self
    }

    /// Whether or not to follow redirects
    pub fn redirects(mut self, redirects: bool) -> Self {
        self.redirects = redirects;
        self
    }
}

impl SchoologyClient {
    /// Creates a new SchoologyClient
    pub fn new(consumer_key: String, consumer_secret: String) -> Self {
        debug!("Creating new SchoologyClient");
        Self {
            consumer_key,
            consumer_secret,
        }
    }

    /// Generates a GET request to the Schoology API (withouth sending it)
    /// Note: Do NOT allow arbitrary paths to be passed in. This allows for path traversal attacks.
    pub async fn get(
        &self,
        path: &str,
        request: SchoologyRequest,
    ) -> Result<reqwest::Response, reqwest::Error> {
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
        let oauth_header = proto::OAuth1AHeader::new(
            "GET".to_string(),
            url.to_string(),
            request.access_token,
            request.token_secret,
        );

        // Generate the signature
        let signature =
            oauth_header.get_header("GET", &url, None, &self.consumer_key, &self.consumer_secret);

        debug!("Generated signature: {}", signature);

        let client = match request.redirects {
            true => reqwest::Client::new(),
            false => match reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
            {
                Ok(client) => client,
                Err(err) => {
                    warn!("Failed to create client: {:?}", err);
                    return Err(err);
                }
            },
        };

        client
            .get(url)
            .header("Accept", "application/json")
            .header("Authorization", signature)
            .send()
            .await
    }
}
