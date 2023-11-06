mod common;

use serde::Deserialize; // TODO: Maybe re-export trait in crate
use serde_json::Value;

#[tokio::test]
async fn test_query_typed() {
    common::initialize().await;

    #[derive(Debug, Deserialize)]
    struct Nation {
        nationkey: u32,
        name: String,
        regionkey: u32,
        comment: String,
    }

    let mut cb = trino::ClientBuilder::default()
        .base_url("http://localhost")
        .port(8080)
        .user("user");
    let client = cb.build();

    let res: Vec<Nation> = client
        .query("SELECT * FROM tpch.tiny.nation")
        .await
        .unwrap();
    assert_eq!(res.len(), 25);
}

#[tokio::test]
async fn test_query_untyped() {
    common::initialize().await;

    let mut cb = trino::ClientBuilder::default()
        .base_url("http://localhost")
        .port(8080)
        .user("user");
    let client = cb.build();

    let res: Vec<Value> = client
        .query("SELECT * FROM tpch.tiny.nation")
        .await
        .unwrap();
    assert_eq!(res.len(), 25);
}
