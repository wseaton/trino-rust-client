use std::path::Path;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::explain::Output;

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
    ) -> Result<Option<NaiveDateTime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        s.map_or(Ok(None), |s| {
            NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S.%f")
                .map(Some)
                .map_err(Error::custom)
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
}

pub fn generate_struct(root_columns: &[Output], file: &Path) -> TokenStream {
    let struct_fields = root_columns.iter().map(|x| {
        let identifier = Ident::new(&x.symbol, Span::call_site());
        let (rust_data_type, attr) = match x.r#type.as_str() {
            "varchar(16383)" => (quote! { Option<String> }, TokenStream::new()),
            "integer" | "int" => (quote! { Option<i32> }, TokenStream::new()),
            "smallint" => (quote! { Option<i16> }, TokenStream::new()),
            "tinyint" => (quote! { Option<i8> }, TokenStream::new()),
            "bigint" => (quote! { Option<i64> }, TokenStream::new()),
            "decimal" | "numeric" => (
                quote! { Option<bigdecimal::BigDecimal> },
                TokenStream::new(),
            ),
            "float" | "real" => (quote! { Option<f32> }, TokenStream::new()),
            "double" => (quote! { Option<f64> }, TokenStream::new()),
            "boolean" | "bool" => (quote! { Option<bool> }, TokenStream::new()),
            "date" => (
                quote! { Option<chrono::NaiveDate> },
                quote! { #[serde(deserialize_with = "dates::from_str_to_naive_date")] },
            ),
            "timestamp" => (
                quote! { Option<chrono::NaiveDateTime> },
                quote! { #[serde(deserialize_with = "dates::from_str_to_naive_datetime")] },
            ),
            "timestamp with time zone" => (
                quote! { Option<chrono::DateTime<chrono::Utc>> },
                quote! { #[serde(deserialize_with = "dates::from_str_to_datetime")] },
            ),
            "time" => (
                quote! { Option<chrono::NaiveTime> },
                quote! { #[serde(deserialize_with = "dates::from_str_to_naive_time")] },
            ),
            "binary" | "varbinary" => (
                quote! { Option<Vec<u8>> },
                quote! {  #[serde(deserialize_with = "binary::from_base64")]},
            ),
            "json" => (quote! { Option<serde_json::Value> }, TokenStream::new()),
            "uuid" => (quote! { Option<uuid::Uuid> }, TokenStream::new()),
            _ if x.r#type.as_str().starts_with("array") => (
                quote! { Option<Vec<array::Element>> },
                quote! {  #[serde(deserialize_with = "array::from_homogeneous_array")]},
            ),
            _ if x.r#type.as_str().starts_with("map") => (
                quote! { Option<std::collections::HashMap<String, serde_json::Value>> },
                TokenStream::new(),
            ),
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
