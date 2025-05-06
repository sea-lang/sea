use std::{collections::HashMap, str::FromStr};

use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum ReefParseError {
    #[error("value parsing error")]
    ValueParseError,
}

pub enum ReefValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

impl FromStr for ReefValue {
    type Err = ReefParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('"') {
            Ok(Self::String(s[1..s.len() - 1].to_string()))
        } else if s == "true" {
            Ok(Self::Boolean(true))
        } else if s == "false" {
            Ok(Self::Boolean(false))
        } else if let Ok(num) = s.parse::<f64>() {
            Ok(Self::Number(num))
        } else {
            Err(ReefParseError::ValueParseError)
        }
    }
}

pub fn parse(code: &str) -> Result<HashMap<String, ReefValue>, ReefParseError> {
    let mut map: HashMap<String, ReefValue> = HashMap::new();

    for line in code.lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') {
            continue;
        }

        let (key, value) = l.split_once('=').unwrap();
        map.insert(key.to_string(), ReefValue::from_str(value)?);
    }

    Ok(map)
}
