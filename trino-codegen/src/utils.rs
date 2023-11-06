use std::path::Path;

use crate::explain::Output;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use regex::Regex;

pub mod array {
    use serde::de::{self, Deserializer};
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    pub enum Element {
        Int(Option<i32>),
        Float(Option<f64>),
        String(Option<String>),
        // TODO: add more types here
    }

    pub fn from_homogeneous_array<'de, D>(deserializer: D) -> Result<Option<Vec<Element>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<Vec<Element>>::deserialize(deserializer)
    }
}

pub mod binary {
    use data_encoding;
    use serde::{
        de::{self, Deserializer},
        Deserialize,
    };

    pub fn from_base64<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(s) => data_encoding::BASE64
                .decode(s.as_bytes())
                .map(Some)
                .map_err(de::Error::custom),
            None => Ok(None),
        }
    }
}

pub mod dates {

    use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
    use serde::de::Error;
    use serde::{Deserialize, Deserializer};

    pub fn from_str_to_naive_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        s.map_or(Ok(None), |s| {
            NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .map(Some)
                .map_err(Error::custom)
        })
    }

    pub fn from_str_to_naive_datetime<'de, D>(
        deserializer: D,
    ) -> Result<Option<chrono::NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        s.map_or(Ok(None), |s| {
            chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f")
                .map(Some)
                .map_err(serde::de::Error::custom)
        })
    }

    pub fn from_str_to_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        s.map_or(Ok(None), |s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| Some(dt.with_timezone(&Utc)))
                .map_err(Error::custom)
        })
    }

    pub fn from_str_to_naive_time<'de, D>(
        deserializer: D,
    ) -> Result<Option<chrono::NaiveTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        s.map_or(Ok(None), |s| {
            chrono::NaiveTime::parse_from_str(&s, "%H:%M:%S%.f")
                .map(Some)
                .map_err(serde::de::Error::custom)
        })
    }
}

pub fn generate_struct(root_columns: &[Output], file: &Path) -> TokenStream {
    let varchar_re = Regex::new(r"^varchar\(\d+\)$").unwrap();
    let decimal_re = Regex::new(r"^decimal\(\d+,\d+\)$").unwrap();
    let timestamp_re = Regex::new(r"^timestamp\(\d+\)$").unwrap();
    let time_re = Regex::new(r"^time\(\d+\)$").unwrap();
    let array_re = Regex::new(r"^array\(.+\)$").unwrap();
    let map_re = Regex::new(r"^map\(.+\)$").unwrap();

    let struct_fields = root_columns.iter().map(|x| {
        let identifier = Ident::new(&x.symbol, Span::call_site());
        let (rust_data_type, attr) = match x.r#type.as_str() {
            _ if varchar_re.is_match(x.r#type.as_str()) => {
                (quote! { Option<String> }, TokenStream::new())
            }
            _ if decimal_re.is_match(x.r#type.as_str()) => (
                quote! { Option<bigdecimal::BigDecimal> },
                TokenStream::new(),
            ),
            _ if timestamp_re.is_match(x.r#type.as_str()) => (
                quote! { Option<chrono::NaiveDateTime> },
                quote! { #[serde(deserialize_with = "dates::from_str_to_naive_datetime")] },
            ),
            _ if time_re.is_match(x.r#type.as_str()) => (
                quote! { Option<chrono::NaiveTime> },
                quote! { #[serde(deserialize_with = "dates::from_str_to_naive_time")] },
            ),
            _ if array_re.is_match(x.r#type.as_str()) => (
                quote! { Option<Vec<array::Element>> },
                quote! {  #[serde(deserialize_with = "array::from_homogeneous_array")]},
            ),
            _ if map_re.is_match(x.r#type.as_str()) => (
                quote! { Option<std::collections::HashMap<String, serde_json::Value>> },
                TokenStream::new(),
            ),

            "integer" | "int" => (quote! { Option<i32> }, TokenStream::new()),
            "smallint" => (quote! { Option<i16> }, TokenStream::new()),
            "tinyint" => (quote! { Option<i8> }, TokenStream::new()),
            "bigint" => (quote! { Option<i64> }, TokenStream::new()),

            "float" | "real" => (quote! { Option<f32> }, TokenStream::new()),
            "double" => (quote! { Option<f64> }, TokenStream::new()),
            "boolean" | "bool" => (quote! { Option<bool> }, TokenStream::new()),
            "date" => (
                quote! { Option<chrono::NaiveDate> },
                quote! { #[serde(deserialize_with = "dates::from_str_to_naive_date")] },
            ),

            "timestamp with time zone" => (
                quote! { Option<chrono::DateTime<chrono::Utc>> },
                quote! { #[serde(deserialize_with = "dates::from_str_to_datetime")] },
            ),

            "binary" | "varbinary" => (
                quote! { Option<Vec<u8>> },
                quote! {  #[serde(deserialize_with = "binary::from_base64")]},
            ),
            "json" => (quote! { Option<serde_json::Value> }, TokenStream::new()),
            "uuid" => (quote! { Option<uuid::Uuid> }, TokenStream::new()),

            _ => (quote! { Option<String> }, TokenStream::new()),
        };
        quote! { #attr pub #identifier: #rust_data_type, }
    });

    let combined_fields = struct_fields.collect::<TokenStream>();

    let structname = Ident::new(
        &format!(
            "{}QueryResult",
            capitalize_first(
                &file
                    .file_stem()
                    .expect("there better be a file stem!")
                    .to_string_lossy()
            )
        ),
        Span::call_site(),
    );

    quote! {
        #[derive(Debug, Deserialize)]
        pub struct #structname {
            #combined_fields
        }
    }
}

pub fn capitalize_first(s: &str) -> String {
    s.chars()
        .take(1)
        .flat_map(|f| f.to_uppercase())
        .chain(s.chars().skip(1))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::explain::ExplainRoot;
    use crate::utils::generate_struct;

    #[test]
    fn test_generate_struct() {
        let file = Path::new("/tmp/file");

        let root: ExplainRoot =
            serde_json::from_str(include_str!("../data/explain2.json")).unwrap();

        let root_columns = &root.outputs;

        let generated_struct = generate_struct(root_columns, file);

        println!("{generated_struct:#?}")
    }
}
