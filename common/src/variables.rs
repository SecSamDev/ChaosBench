use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};

use crate::{common::deserialize_null_default, parameters::TestParameter};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
#[serde(transparent)]
pub struct TestVariables(pub BTreeMap<String, TestParameter>);

#[derive(Clone, Debug, Default, Serialize, Deserialize, Hash)]
pub struct ScenarioVariables {
    #[serde(flatten)]
    pub global : TestVariables,
    #[serde(default, deserialize_with="deserialize_null_default")]
    pub windows : TestVariables,
    #[serde(default, deserialize_with="deserialize_null_default")]
    pub linux : TestVariables
}

impl TestVariables {
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
}

impl From<&ScenarioVariables> for TestVariables {
    fn from(value: &ScenarioVariables) -> Self {
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