pub mod response;

use data_encoding::BASE64;
use reqwest::{Client as ReqwestClient, Response};

use response::*;
use serde::de::DeserializeOwned;
use tokio::time::Duration;

use sha2::{Digest, Sha256};

use tracing::debug;
use tracing::instrument;

use serde_json::Value;

// Helper function to hash a string
fn hash_string(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub struct ClientBuilder {
    base_url: Option<String>,
    port: Option<u32>,
    user: Option<String>,
    password: Option<String>,
    timeout: Option<Duration>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        let port: u32 = std::env::var("TRINO_PORT")
            .expect("TRINO_PORT must be set.")
            .parse()
            .expect("TRINO_PORT must be an integer.");

        ClientBuilder {
            base_url: std::env::var("TRINO_HOST").ok(),
            port: Some(port),
            user: std::env::var("TRINO_USER").ok(),
            password: std::env::var("TRINO_PASSWORD").ok(),
            timeout: None,
        }
    }
}

impl ClientBuilder {
    pub fn new() -> Self {
        ClientBuilder {
            base_url: None,
            port: None,
            user: None,
            password: None,
            timeout: None,
        }
    }

    pub fn base_url(mut self, url: &str) -> Self {
        self.base_url = Some(url.to_owned());
        self
    }

    pub fn port(mut self, port: u32) -> Self {
        self.port = Some(port);
        self
    }

    pub fn user(mut self, user: &str) -> Self {
        self.user = Some(user.to_owned());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = Some(password.to_owned());
        self
    }

    pub fn build(self) -> Client {
        let mut cb = ReqwestClient::builder();

        if let Some(timeout) = self.timeout {
            cb = cb.timeout(timeout);
        }

        if let (Some(username), Some(password)) = (&self.user, self.password) {
            let encoded = BASE64.encode(format!("{}:{}", username, password).as_bytes());
            let auth_header = format!("Basic {}", &encoded);

            cb = cb.default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert(
                    reqwest::header::AUTHORIZATION,
                    reqwest::header::HeaderValue::from_str(&auth_header).unwrap(),
                );
                headers
            });
        }

        let http_client = cb.build().expect("Failed to build reqwest client");

        Client {
            base_url: self.base_url.expect("Base URL must be set."),
            port: self.port.expect("Port must be set."),
            user: self.user,
            http_client,
        }
    }
}

pub struct Client {
    pub base_url: String,
    pub port: u32,
    pub user: Option<String>,
    pub http_client: ReqwestClient,
}

impl Client {
    // TODO:
    //  - Implement query cancellation i.e. DELETE to nextUri
    //  - Implement own errors
    //  - Implement paging
    #[instrument(skip(self, query_str), fields(correlation_id = %hash_string(query_str)))]
    pub async fn query<T>(&self, query_str: &str) -> Result<Vec<T>, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        let mut response = self.initial_request(query_str).await?;
        let headers = response.headers().clone();
        debug!("trino response headers: {:#?}", headers);
        let raw_text = response.text().await?;
        debug!("raw_text: {}", raw_text);

        let mut response_body: QueryResults = serde_json::from_str(&raw_text).unwrap();
        debug!("initial response_body: {:?}", response_body);

        let mut data = Vec::new();
        while let Some(next_uri) = response_body.next_uri {
            response = self.next_request(&next_uri).await?;
            response_body = response.json().await?;

            if let Some(rows) = response_body.data {
                debug!("rows: {:?}", rows);
                data.extend(rows.into_iter().map(|x| serde_json::from_value(x).unwrap()));
            }
        }
        Ok(data)
    }

    #[instrument(skip(self, query_str), fields(correlation_id = %hash_string(query_str)))]
    pub async fn query_once<T>(&self, query_str: &str) -> Result<T, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        let mut response = self.initial_request(query_str).await?;
        let headers = response.headers().clone();
        debug!("trino response headers: {:#?}", headers);
        let raw_text = response.text().await?;
        debug!("raw_text: {}", raw_text);

        let mut response_body: QueryResults = serde_json::from_str(&raw_text).unwrap();
        debug!("initial response_body: {:?}", response_body);

        while let Some(next_uri) = response_body.next_uri {
            response = self.next_request(&next_uri).await?;
            response_body = response.json().await?;

            if let Some(mut data) = response_body.data {
                if let Some(Value::Array(arr)) = data.pop() {
                    debug!("array: {:#?}", arr);
                    // pop the first element off the array, which is the data we want
                    let json_value: T = serde_json::from_str(
                        arr[0].clone().as_str().expect("This should be a string!"),
                    )
                    .expect("Failed to parse JSON");
                    return Ok(json_value);
                }
            }
        }
        panic!("Failed to get JSON formatted response from Trino");
    }

    // Make the initial request to Trino, hash the query string as a correlation_id
    #[instrument(skip(self, query_str), fields(correlation_id = %hash_string(query_str)))]
    async fn initial_request(&self, query_str: &str) -> Result<Response, reqwest::Error> {
        let conn_str = format!("{}:{}/v1/statement", &self.base_url, &self.port);
        let mut rb = self.http_client.post(conn_str).body(query_str.to_string());

        if let Some(user) = &self.user {
            rb = rb.header("X-Trino-User", user);
        }

        debug!("initial_request headers: {:#?}", rb);

        rb.send().await
    }

    #[tracing::instrument(skip(self))]
    async fn next_request(&self, next_uri: &str) -> Result<Response, reqwest::Error> {
        debug!("navigating to next_uri: {}", next_uri);
        self.http_client.get(next_uri).send().await
    }
}
