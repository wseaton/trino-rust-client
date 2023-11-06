use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt::Display, str::FromStr};

fn nan_as_none<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr + Deserialize<'de> + Display, // Added Deserialize trait bound
    T::Err: Display,
    D: Deserializer<'de>,
{
    let value = T::deserialize(deserializer).ok();
    match value {
        Some(val) => {
            if val.to_string().to_lowercase() == "nan" {
                Ok(None)
            } else {
                Ok(Some(val))
            }
        }
        None => Ok(None),
    }
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ExplainRoot {
    pub id: String,
    pub name: String,
    pub descriptor: Descriptor,
    pub outputs: Vec<Output>,
    pub details: Vec<Detail>,
    pub estimates: Vec<Estimate>,
    pub children: Vec<Child>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Descriptor {
    #[serde(rename = "columnNames")]
    pub column_names: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
    pub symbol: String,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Detail {
    String(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Estimate {
    #[serde(rename = "outputRowCount")]
    #[serde(deserialize_with = "nan_as_none")]
    pub output_row_count: Option<f64>,

    #[serde(rename = "outputSizeInBytes")]
    #[serde(deserialize_with = "nan_as_none")]
    pub output_size_in_bytes: Option<f64>,

    #[serde(rename = "cpuCost")]
    #[serde(deserialize_with = "nan_as_none")]
    pub cpu_cost: Option<f32>,

    #[serde(rename = "memoryCost")]
    #[serde(deserialize_with = "nan_as_none")]
    pub memory_cost: Option<f32>,

    #[serde(rename = "networkCost")]
    #[serde(deserialize_with = "nan_as_none")]
    pub network_cost: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Child {
    pub id: String,
    pub name: String,
    pub descriptor: ChildDescriptor,
    pub outputs: Vec<Output>,
    pub details: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChildDescriptor {
    pub table: Option<String>,
}

#[cfg(test)]
mod tests {

    use crate::explain::ExplainRoot;

    #[test]
    fn test_deserialize() {
        let data: String = include_str!("../data/explain.json").to_string();

        let _root: ExplainRoot = serde_json::from_str(&data).expect("Fail to deserialize");
    }

    #[test]
    fn test_deserialize2() {
        let data: String = include_str!("../data/explain2.json").to_string();

        let _root: ExplainRoot = serde_json::from_str(&data).expect("Fail to deserialize");
    }
}
