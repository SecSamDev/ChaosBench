use std::fmt::Display;

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

impl From<ChaosError> for String {
    fn from(value: ChaosError) -> Self {
        match value {
            ChaosError::Other(v) => v,
            ChaosError::Unknown => "Unknown error".into(),
        }
    }
}

impl Display for ChaosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChaosError::Other(v) => f.write_str(&v),
            ChaosError::Unknown => f.write_str("Unknown error"),
        }
    }
}