use loco_rs::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Settings {
    pub frontend: String,
    pub backend: String,
}

impl Settings {
    pub fn from_opt_json(value: &Option<serde_json::Value>) -> Result<Self> {
        match value.clone() {
            Some(val) => Ok(serde_json::from_value(val)?),
            None => Err(loco_rs::Error::InternalServerError),
        }
    }
    pub fn from_json(value: &serde_json::Value) -> Result<Self> {
        Ok(serde_json::from_value(value.clone())?)
    }
}
