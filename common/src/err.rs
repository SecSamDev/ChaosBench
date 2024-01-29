use std::default;

use serde::{Serialize, Deserialize};


pub type ChaosResult<T> = Result<T, ChaosError>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum ChaosError {
    Other(String),
    #[default]
    Unknown
}

impl From<String> for ChaosError {
    fn from(value: String) -> Self {
        ChaosError::Other(value)
    }
}