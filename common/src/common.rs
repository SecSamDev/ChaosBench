use std::{fmt, hash::{Hash, Hasher}, time::Duration};

use serde::{de::Visitor, Deserialize, Deserializer, Serializer};

use crate::{action::CustomAction, parameters::ScenarioParameters, variables::ScenarioVariables};

pub fn hash_params_and_actions(params : &ScenarioParameters, actions : &[CustomAction], variables : &ScenarioVariables) -> u64 {
    let mut state = std::collections::hash_map::DefaultHasher::new();
    params.hash(&mut state);
    variables.hash(&mut state);
    for action in actions {
        action.hash(&mut state);
    }
    state.finish()
}

pub fn deserialize_null_default<'de, D, T>(d: D) -> Result<T, D::Error>
    where 
        D: Deserializer<'de>,
        T: Default + Deserialize<'de>
{
    let opt = Option::deserialize(d)?;
    Ok(opt.unwrap_or_default())
}

pub fn default_timeout() -> Duration {
    Duration::from_secs(30)
}

pub fn deserialize_duration<'de, D>(d: D) -> Result<Duration, D::Error>
    where 
        D: Deserializer<'de>
{
    match d.deserialize_str(StrDurationVisitor) {
        Ok(v) => Ok(v),
        Err(_) => Ok(default_timeout())
    }
}

pub fn serialize_duration<S>(v: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{}s", v.as_secs()))
}

struct StrDurationVisitor;

impl<'de> Visitor<'de> for StrDurationVisitor {
    type Value = Duration;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid string duration")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error, {
            string_to_duration(v).ok_or(serde::de::Error::custom("Not a valid Duration string. Format 30s, 30m"))
    }
}

pub fn string_to_duration(v : &str) -> Option<Duration> {
    let v = v.trim();
    if v.len() == 1 {
        let chr1 = v.chars().next()?;
        if !chr1.is_numeric() {
            return None
        }
        return Some(Duration::from_secs(chr1.to_digit(10)? as u64))
    }
    let last_char = v.chars().next_back()?;
    let (modifier, duration) = if last_char.is_numeric() {
        (1, v.parse::<u64>().unwrap_or(1))
    }else{
        (modifier_from_letter(last_char), v[0..v.len() - 1].parse::<u64>().unwrap_or(1))
    };
    Some(Duration::from_secs(duration * modifier))
}
fn modifier_from_letter(letter : char) -> u64 {
    match letter {
        's' => 1,
        'm' => 60,
        'h' => 3600,
        _ => 1
    }
}

#[test]
fn should_parse_duration() {
    assert_eq!(Duration::from_secs(30), string_to_duration("30s").unwrap());
    assert_eq!(Duration::from_secs(30), string_to_duration("30").unwrap());
    assert_eq!(Duration::from_secs(60), string_to_duration("1m").unwrap());
    assert_eq!(Duration::from_secs(3600), string_to_duration("1h").unwrap());
}