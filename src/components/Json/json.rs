use std::collections::HashMap;

//Serialize 직렬화 인데 결과값 차이
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, from_str, to_string, to_value};
//Json 반환기
pub trait JsonConverter {
    fn json_to_object<T: DeserializeOwned>(&self, json: &str) -> Result<T, Box<dyn std::error::Error>>;

    fn json_to_object_safe<T: DeserializeOwned + Default>(&self, json: &str) -> Result<T, Box<dyn std::error::Error>>;
}

pub struct JsonService;

impl JsonConverter for JsonService {
    fn json_to_object<T: DeserializeOwned>(&self, json: &str) -> Result<T, Box<dyn std::error::Error>> {
        let result: T = from_str(json)?;
        Ok(result)
    }

    fn json_to_object_safe<T: DeserializeOwned + Default>(&self, json: &str) -> Result<T, Box<dyn std::error::Error>> {
        Ok(from_str(json).unwrap_or_default())
    }
}