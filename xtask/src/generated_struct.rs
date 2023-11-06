use serde::Deserialize;
use trino_codegen::utils::array;
use trino_codegen::utils::binary;
use trino_codegen::utils::dates;
#[derive(Debug, Deserialize)]
pub struct Generated_structQueryResult {
    pub expr: Option<i32>,
    pub expr_0: Option<i16>,
    pub expr_1: Option<i8>,
    pub expr_2: Option<bigdecimal::BigDecimal>,
    pub expr_3: Option<f32>,
    pub expr_4: Option<f64>,
    pub expr_5: Option<bool>,
    #[serde(deserialize_with = "dates::from_str_to_naive_date")]
    pub expr_6: Option<chrono::NaiveDate>,
    #[serde(deserialize_with = "dates::from_str_to_naive_datetime")]
    pub expr_7: Option<chrono::NaiveDateTime>,
    pub expr_8: Option<String>,
    #[serde(deserialize_with = "dates::from_str_to_naive_time")]
    pub expr_9: Option<chrono::NaiveTime>,
    pub expr_10: Option<String>,
    pub expr_11: Option<String>,
    #[serde(deserialize_with = "binary::from_base64")]
    pub expr_12: Option<Vec<u8>>,
    pub expr_13: Option<serde_json::Value>,
    #[serde(deserialize_with = "array::from_homogeneous_array")]
    pub expr_14: Option<Vec<array::Element>>,
    pub map: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub expr_15: Option<uuid::Uuid>,
}
