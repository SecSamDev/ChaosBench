use std::{collections::BTreeMap, fmt, time::Duration};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::common::string_to_duration;

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
pub struct TestParameters {
    #[serde(flatten)]
    pub(crate) parameters: BTreeMap<String, TestParameter>,
}

impl TestParameters {
    pub fn new() -> Self {
        Self {
            parameters: BTreeMap::new(),
        }
    }
    pub fn inner(&self) -> &BTreeMap<String, TestParameter> {
        &self.parameters
    }
    pub fn get(&self, name: &str) -> Option<&TestParameter> {
        self.parameters.get(name)
    }
    pub fn insert(&mut self, name: &str, value: TestParameter) {
        self.parameters.insert(name.into(), value);
    }
    pub fn contains_key(&self, name: &str) -> bool {
        self.parameters.contains_key(name)
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub enum TestParameter {
    Text(String),
    Bool(bool),
    U64(u64),
    I64(i64),
    F64(f64),
    Obj(BTreeMap<String, TestParameter>),
    Vec(Vec<TestParameter>),
    #[default]
    Null,
}
struct TestParameterVisitor;

impl<'de> Visitor<'de> for TestParameterVisitor {
    type Value = TestParameter;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid parameter type")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::Text(v.into()))
    }
    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::Text(v))
    }
    fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::I64(value.into()))
    }
    fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::I64(value.into()))
    }

    fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::I64(value.into()))
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::I64(value.into()))
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::U64(value.into()))
    }
    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::U64(value.into()))
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::U64(value.into()))
    }
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::U64(value.into()))
    }
    fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::Bool(v))
    }
    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
        where
            E: de::Error, {
                Ok(TestParameter::Text(v.to_string()))
    }
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
        where
            E: de::Error, {
        Ok(TestParameter::F64(v.into()))
    }
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error, {
        Ok(TestParameter::F64(v.into()))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error, {
        Ok(TestParameter::Null)
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>, {
        let mut vc = Vec::with_capacity(seq.size_hint().unwrap_or(32));
        loop {
            let element : TestParameter = match seq.next_element() {
                Ok(Some(v)) => v,
                _ => break
            };
            vc.push(element);
        }
        Ok(TestParameter::Vec(vc))
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>, {
        let mut obj = BTreeMap::new();
        loop {
            let (key, value) : (String, TestParameter) = match map.next_entry() {
                Ok(Some(v)) => v,
                _ => break
            };
            obj.insert(key, value);
        }
        Ok(TestParameter::Obj(obj))
    }
}

impl<'de> Deserialize<'de> for TestParameter {
    fn deserialize<D>(deserializer: D) -> Result<TestParameter, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(TestParameterVisitor)
    }
}

impl TryFrom<&TestParameter> for String {
    type Error = &'static str;
    fn try_from(value: &TestParameter) -> Result<Self,Self::Error> {
        Ok(match value {
            TestParameter::Text(v) => v.into(),
            TestParameter::Bool(v) => v.to_string(),
            TestParameter::U64(v) => v.to_string(),
            TestParameter::I64(v) => v.to_string(),
            TestParameter::F64(v) => v.to_string(),
            TestParameter::Obj(_) => return Err("Cannot convert from obj to string"),
            TestParameter::Vec(_) => return Err("Cannot convert from vec to string"),
            TestParameter::Null => "".into()
        })
    }
}

impl<'a> TryFrom<&'a TestParameter> for &'a str {
    type Error = &'static str;
    fn try_from(value: &'a TestParameter) -> Result<Self,Self::Error> {
        match value {
            TestParameter::Text(v) => return Ok(v.as_str()),
            TestParameter::Null => return Ok(""),
            _ => Err("Cannot convert to &str")
        }
    }
}
impl<'a> TryFrom<&'a TestParameter> for &'a BTreeMap<String, TestParameter> {
    type Error = &'static str;
    fn try_from(value: &'a TestParameter) -> Result<Self,Self::Error> {
        match value {
            TestParameter::Obj(v) => Ok(v),
            _ => Err("Cannot convert to Map")
        }
    }
}

impl TryFrom<&TestParameter> for i32 {
    type Error = &'static str;
    fn try_from(value: &TestParameter) -> Result<Self,Self::Error> {
        Ok(match value {
            TestParameter::U64(v) => *v as i32,
            TestParameter::I64(v) => *v as i32,
            _ => return Err("Invalid numeric value")
        })
    }
}
impl TryFrom<TestParameter> for i32 {
    type Error = &'static str;
    fn try_from(value: TestParameter) -> Result<Self,Self::Error> {
        Ok(match value {
            TestParameter::U64(v) => v as i32,
            TestParameter::I64(v) => v as i32,
            _ => return Err("Invalid numeric value")
        })
    }
}

impl TryFrom<&TestParameter> for Duration {
    type Error = &'static str;
    fn try_from(value: &TestParameter) -> Result<Self,Self::Error> {
        Ok(match value {
            TestParameter::U64(v) => Duration::from_secs(*v),
            TestParameter::I64(v) => Duration::from_secs(*v as u64),
            TestParameter::Text(v) => string_to_duration(v).ok_or("Invalid duration string")?,
            _ => return Err("Invalid duration value")
        })
    }
}

impl std::hash::Hash for TestParameter {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            TestParameter::Text(v) => v.hash(state),
            TestParameter::Bool(v) => v.hash(state),
            TestParameter::U64(v) => v.hash(state),
            TestParameter::I64(v) => v.hash(state),
            TestParameter::F64(v) => {
                state.write(&v.to_le_bytes());
            }
            TestParameter::Obj(map) => {
                for (k,v) in map {
                    state.write(k.as_bytes());
                    v.hash(state);
                }
            }
            TestParameter::Vec(v) => {
                for param in v {
                    param.hash(state);
                }
            }
            TestParameter::Null => state.write_u8(0)
        }
    }
}