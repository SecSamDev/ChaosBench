use std::{collections::BTreeMap, fmt, time::Duration};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
    ser::{SerializeSeq, SerializeMap}
};

use crate::{common::{deserialize_null_default, string_to_duration}, variables::TestVariables};

pub const REMOTE_SERVER : &str = "remote_server";

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
pub struct ScenarioParameters {
    #[serde(flatten)]
    pub global : TestParameters,
    #[serde(default, deserialize_with="deserialize_null_default")]
    pub windows : TestParameters,
    #[serde(default, deserialize_with="deserialize_null_default")]
    pub linux : TestParameters
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct TestParameters(pub BTreeMap<String, TestParameter>);


impl TestParameters {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn inner(&self) -> &BTreeMap<String, TestParameter> {
        &self.0
    }
    pub fn get(&self, name: &str) -> Option<&TestParameter> {
        self.0.get(name)
    }
    pub fn insert(&mut self, name: &str, value: TestParameter) {
        self.0.insert(name.into(), value);
    }
    pub fn contains_key(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }
    pub fn replace_with_vars(&mut self, vars : &TestVariables) {
        for (_, v) in self.0.iter_mut() {
            v.replace_with_vars(vars);
        }
    }
}

impl From<&ScenarioParameters> for TestParameters {
    fn from(value: &ScenarioParameters) -> Self {
        let mut params = Self::new();
        for (k, v) in &value.global.0 {
            params.insert(k, v.clone());
        }
        #[cfg(target_os="windows")]
        for (k, v) in &value.windows.0 {
            params.insert(k, v.clone());
        }
        #[cfg(target_os="linux")]
        for (k, v) in &value.linux.0 {
            params.insert(k, v.clone());
        }
        params
    }
}

#[derive(Clone, Debug, Default)]
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
        E: de::Error,
    {
        Ok(TestParameter::Text(v.to_string()))
    }
    fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::F64(v.into()))
    }
    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::F64(v.into()))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(TestParameter::Null)
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut vc = Vec::with_capacity(seq.size_hint().unwrap_or(32));
        loop {
            let element: TestParameter = match seq.next_element() {
                Ok(Some(v)) => v,
                _ => break,
            };
            vc.push(element);
        }
        Ok(TestParameter::Vec(vc))
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        let mut obj = BTreeMap::new();
        loop {
            let (key, value): (String, TestParameter) = match map.next_entry() {
                Ok(Some(v)) => v,
                _ => break,
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
    fn try_from(value: &TestParameter) -> Result<Self, Self::Error> {
        Ok(match value {
            TestParameter::Text(v) => v.into(),
            TestParameter::Bool(v) => v.to_string(),
            TestParameter::U64(v) => v.to_string(),
            TestParameter::I64(v) => v.to_string(),
            TestParameter::F64(v) => v.to_string(),
            TestParameter::Obj(_) => return Err("Cannot convert from obj to string"),
            TestParameter::Vec(_) => return Err("Cannot convert from vec to string"),
            TestParameter::Null => "".into(),
        })
    }
}

impl<'a> TryFrom<&'a TestParameter> for &'a str {
    type Error = &'static str;
    fn try_from(value: &'a TestParameter) -> Result<Self, Self::Error> {
        match value {
            TestParameter::Text(v) => return Ok(v.as_str()),
            TestParameter::Null => return Ok(""),
            _ => Err("Cannot convert to &str"),
        }
    }
}
impl<'a> TryFrom<&'a TestParameter> for &'a BTreeMap<String, TestParameter> {
    type Error = &'static str;
    fn try_from(value: &'a TestParameter) -> Result<Self, Self::Error> {
        match value {
            TestParameter::Obj(v) => Ok(v),
            _ => Err("Cannot convert to Map"),
        }
    }
}

impl TryFrom<&TestParameter> for i32 {
    type Error = &'static str;
    fn try_from(value: &TestParameter) -> Result<Self, Self::Error> {
        Ok(match value {
            TestParameter::U64(v) => *v as i32,
            TestParameter::I64(v) => *v as i32,
            _ => return Err("Invalid numeric value"),
        })
    }
}
impl TryFrom<TestParameter> for i32 {
    type Error = &'static str;
    fn try_from(value: TestParameter) -> Result<Self, Self::Error> {
        Ok(match value {
            TestParameter::U64(v) => v as i32,
            TestParameter::I64(v) => v as i32,
            _ => return Err("Invalid numeric value"),
        })
    }
}

impl TryFrom<&TestParameter> for Duration {
    type Error = &'static str;
    fn try_from(value: &TestParameter) -> Result<Self, Self::Error> {
        Ok(match value {
            TestParameter::U64(v) => Duration::from_secs(*v),
            TestParameter::I64(v) => Duration::from_secs(*v as u64),
            TestParameter::Text(v) => string_to_duration(v).ok_or("Invalid duration string")?,
            _ => return Err("Invalid duration value"),
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
                for (k, v) in map {
                    state.write(k.as_bytes());
                    v.hash(state);
                }
            }
            TestParameter::Vec(v) => {
                for param in v {
                    param.hash(state);
                }
            }
            TestParameter::Null => state.write_u8(0),
        }
    }
}

impl Serialize for TestParameter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TestParameter::Text(v) => serializer.serialize_str(v),
            TestParameter::Bool(v) => serializer.serialize_bool(*v),
            TestParameter::U64(v) => serializer.serialize_u64(*v),
            TestParameter::I64(v) => serializer.serialize_i64(*v),
            TestParameter::F64(v) => serializer.serialize_f64(*v),
            TestParameter::Obj(m) => {
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            },
            TestParameter::Vec(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for element in v {
                    seq.serialize_element(element)?;
                }
                seq.end()
            },
            TestParameter::Null => serializer.serialize_none(),
        }
    }
}

impl TestParameter {
    pub fn replace_with_vars(&mut self, vars : &TestVariables) {
        match self {
            TestParameter::Text(v) => interpolate_text(v, vars),
            TestParameter::Obj(obj) => {
                for (_, p) in obj {
                    p.replace_with_vars(vars);
                }
            },
            TestParameter::Vec(v) => {
                for p in v {
                    p.replace_with_vars(vars);
                }
            },
            _ => return
        }
    }
}

pub fn interpolate_text(template : &mut String, vars : &TestVariables) {
    loop {
        let (start, end, variable) = match get_next_variable_position(template) {
            Some(v) => v,
            None => break
        };
        let value = match vars.0.get(variable) {
            Some(v) => v,
            None => break
        };
        let txt : &str = match value.try_into() {
            Ok(v) => v,
            Err(_) => break
        };
        let start_part = if start == 0 { "" } else { &template[0..start] };
        let end_part = if end - start == 0 { "" } else { &template[end..] };
        *template = format!("{}{}{}", start_part, txt, end_part);
        println!("{}", template);
    }
}

fn get_next_variable_position(template : &str) -> Option<(usize, usize, &str)> {
    let position = template.find("${")?;
    let end_position = template[position + 2..].find('}')? + position + 2;
    let var = &template[position + 2..end_position];
    Some((position, end_position + 1, var))
}

#[test]
fn should_interpolate_parameters() {
    let file_content = std::fs::read_to_string("./src/basic_scenario.yaml").unwrap();
    let basic_scene : crate::scenario::TestScenario = serde_yaml::from_str(&file_content).unwrap();
    let mut parameters : TestParameters = (&basic_scene.actions.get(0).unwrap().parameters).into();
    let variables: TestVariables  = (&basic_scene.variables).into();
    parameters.replace_with_vars(&variables);
    let command : &str = parameters.get("command").unwrap().try_into().unwrap();
    assert_eq!("C:\\Program Files\\program\\uninstaller.exe --force", command);
}