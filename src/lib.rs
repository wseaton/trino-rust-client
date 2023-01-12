pub mod response;

use std::io::Read;

use flate2::read::GzDecoder;
use reqwest::blocking::{Client as ReqwestClient, Response};

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
    pub fn query<T>(&self, query_str: &str) -> Result<Vec<T>, reqwest::Error>
    where
        T: DeserializeOwned,
    {
        let mut response = self.initial_request(query_str)?;
        let headers = response.headers().clone();
        debug!("trino response headers: {:#?}", headers);
        let raw_text = response.text()?;

        // decompress the text if gzip
        let raw_text = if headers.get("Content-Encoding") == Some(&"gzip".parse().unwrap()) {
            let mut decoder = GzDecoder::new(raw_text.as_bytes());
            let mut decoded_text = String::new();
            decoder.read_to_string(&mut decoded_text).unwrap();
            decoded_text
        } else {
            raw_text
        };

        debug!("raw_text: {}", raw_text);
        let mut response_body: QueryResults = serde_json::from_str(&raw_text).unwrap();

        debug!("initial response_body: {:?}", response_body);

        let mut data = Vec::new();
        while let Some(next_uri) = response_body.next_uri {
            response = self.next_request(&next_uri)?;
            response_body = response.json()?;
            if let Some(rows) = response_body.data {
                debug!("rows: {:?}", rows);
                data.extend(rows.into_iter().map(|x| serde_json::from_value(x).unwrap()));
            }
        }
        Ok(data)
    }

    fn initial_request(&self, query_str: &str) -> Result<Response, reqwest::Error> {
        let conn_str = format!("{}:{}/v1/statement", &self.base_url, &self.port);
        let mut rb = self.http_client.post(conn_str).body(query_str.to_string());

        if let Some(user) = &self.user {
            rb = rb.header("X-Trino-User", user);
        }

        rb.send()
    }

    fn next_request(&self, next_uri: &str) -> Result<Response, reqwest::Error> {
        self.http_client.get(next_uri).send()
    }
}
