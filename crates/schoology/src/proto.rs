use hmac::{Hmac, Mac};
use sha2::Sha256;
use url::Url;
use urlencoding;
// TODO: we should prob switch to the secure version when it releases
use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OAuth1AHeader {
    /* Stuff for the signature */
    /// The request method, e.g. "GET" or "POST"
    pub request_method: String,
    /// The full request URL, e.g. "https://api.schoology.com/v1/users/me"
    pub request_url: String,

    /* OAuth 1.0a parameters */
    /// Optional: OAuth 1.0a access token
    pub access_token: Option<String>,
    /// Optional: OAuth 1.0a token secret
    pub access_token_secret: Option<String>,
    /// OAuth 1.0a nonce
    pub oauth_nonce: String,
    /// OAuth 1.0a timestamp
    pub oauth_timestamp: String,
}

impl OAuth1AHeader {
    /// Creates a new OAuth 1.0a header
    pub fn new(
        request_method: String,
        request_url: String,
        access_token: Option<String>,
        access_token_secret: Option<String>,
    ) -> Self {
        Self {
            request_method,
            request_url,
            access_token,
            access_token_secret,
            oauth_nonce: Uuid::new_v4().to_string(),
            oauth_timestamp: chrono::Utc::now().timestamp().to_string(),
        }
    }

    /// Gets the OAuth 1.0a header for the request
    fn get_params(&self, consumer_key: &str) -> Vec<(String, String)> {
        let mut params = vec![
            ("oauth_consumer_key", consumer_key),
            ("oauth_signature_method", "HMAC-SHA256"),
            ("oauth_timestamp", &self.oauth_timestamp),
            ("oauth_nonce", &self.oauth_nonce),
            ("oauth_version", "1.0"),
        ];

        if let Some(token) = &self.access_token {
            params.push(("oauth_token", token));
        }

        // Convert the parameters to owned strings
        params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<(String, String)>>()
    }

    /// You probably want `get_header` instead.
    pub fn generate_signature(
        &self,
        request_method: &str,
        url: &Url,
        body_params: Option<Vec<(&str, &str)>>,
        consumer_key: &str,
        consumer_secret: &str,
    ) -> String {
        let mut params = self.get_params(consumer_key);

        // Push any additional parameters
        if let Some(p) = body_params {
            params.extend(p.iter().map(|(k, v)| (k.to_string(), v.to_string())));
        }

        let mut url = url.clone();

        // Push any additional parameters from the URL query string
        let pairs = url
            .query_pairs()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<Vec<(String, String)>>();

        params.extend(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())));

        // Sort the parameters by key and if duplicate keys exist, sort by value
        params.sort_by(|a, b| {
            if a.0 == b.0 {
                a.1.cmp(&b.1)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // URL encode the parameters and join them with "&"
        let params = params
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    urlencoding::encode(k).to_string(),
                    urlencoding::encode(v).to_string()
                )
            })
            .collect::<Vec<String>>()
            .join("&");

        // URL encode the params
        let params = urlencoding::encode(&params).to_string();

        // Strip the url of any query parameters, fragment, etc.
        url.set_query(None);
        url.set_fragment(None);
        let url = urlencoding::encode(&url.to_string()).to_string();

        // `http method + "&" + url + "&" + params`
        let mut param_string =
            String::with_capacity(request_method.len() + url.len() + params.len() + 2);
        param_string.push_str(&request_method);
        param_string.push_str("&");
        param_string.push_str(&url);
        param_string.push_str("&");
        param_string.push_str(&params);

        // Generate the signing key `consumer_secret + "&" + access_token_secret`
        let mut signing_key = String::with_capacity(
            consumer_secret.len()
                + self
                    .access_token_secret
                    .clone()
                    .unwrap_or("".to_string())
                    .len()
                + 1,
        );
        signing_key.push_str(&consumer_secret);
        signing_key.push_str("&");
        signing_key.push_str(&self.access_token_secret.clone().unwrap_or("".to_string()));

        debug!("Signing key: {}", param_string);

        // Generate the signature
        let mut mac = match Hmac::<Sha256>::new_from_slice(signing_key.as_bytes()) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to generate HMAC-SHA256 signature: {}", e);
                return "".to_string();
            }
        };
        mac.update(param_string.as_bytes());

        // Base64 encode the signature
        STANDARD_NO_PAD.encode(&mac.finalize().into_bytes())
    }

    /// Gets the OAuth 1.0a header for the request
    pub fn get_header(
        &self,
        request_method: &str,
        url: &Url,
        body_params: Option<Vec<(&str, &str)>>,
        consumer_key: &str,
        consumer_secret: &str,
    ) -> String {
        // First let's generate the signature
        let signature = self.generate_signature(
            request_method,
            url,
            body_params,
            consumer_key,
            consumer_secret,
        );

        // Now let's generate the header
        let mut header = self.get_params(consumer_key);

        // Push the signature
        header.push(("oauth_signature".to_string(), signature.to_string()));

        // Sort the parameters by key and if duplicate keys exist, sort by value
        header.sort_by(|a, b| {
            if a.0 == b.0 {
                a.1.cmp(&b.1)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // URL encode the parameters and join them with ","
        let header_str = header
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}=\"{}\"",
                    urlencoding::encode(k).to_string(),
                    urlencoding::encode(v).to_string()
                )
            })
            .collect::<Vec<String>>()
            .join(",");

        // Add the "OAuth " prefix
        let mut header = String::with_capacity(header.len() + 6);
        header.push_str("OAuth ");
        header.push_str(&header_str);

        header
    }
}
