pub mod response;

use reqwest::{Client as ReqwestClient, Response};

use response::*;
use serde::de::DeserializeOwned;

use tracing::debug;

pub struct Client {
    pub base_url: String,
    pub port: u32,
    pub user: Option<String>,
    pub http_client: ReqwestClient,
}

impl Client {
    pub fn new(base_url: &str, port: u32, user: Option<&str>) -> Client {
        Client {
            base_url: base_url.to_string(),
            port,
            user: user.map(|user| user.to_owned()),
            http_client: ReqwestClient::new(),
        }
    }

    // TODO:
    //  - Implement query cancellation i.e. DELETE to nextUri
    //  - Implement own errors
    //  - Implement paging
    //  - Add client builder
    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self, query_str))]
    async fn initial_request(&self, query_str: &str) -> Result<Response, reqwest::Error> {
        let conn_str = format!("{}:{}/v1/statement", &self.base_url, &self.port);
        let mut rb = self.http_client.post(conn_str).body(query_str.to_string());

        if let Some(user) = &self.user {
            rb = rb.header("X-Trino-User", user);
        }

        rb.send().await
    }

    #[tracing::instrument(skip(self))]
    async fn next_request(&self, next_uri: &str) -> Result<Response, reqwest::Error> {
        debug!("navigating to next_uri: {}", next_uri);
        self.http_client.get(next_uri).send().await
    }
}
