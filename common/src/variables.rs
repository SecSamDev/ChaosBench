use std::collections::BTreeMap;

use serde::{Serialize, Deserialize};

use crate::{api::agent::{native_arch_str, Os}, common::deserialize_null_default, parameters::TestParameter};

pub const ARCH_VAR : &str = "arch";
pub const OS_VAR : &str = "os";
pub const HOSTNAME_VAR : &str = "hostname";

#[cfg(not(target_os="windows"))]
pub const HOSTNAME_ENV_VAR : &str = "HOSTNAME";
#[cfg(target_os="windows")]
pub const HOSTNAME_ENV_VAR : &str = "COMPUTERNAME";


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
        let mut map = BTreeMap::new();
        map.insert(ARCH_VAR.into(), native_arch_str().to_string().into());
        map.insert(OS_VAR.into(), Into::<&str>::into(Os::default()).to_string().into());
        if let Ok(v) = std::env::var(HOSTNAME_ENV_VAR) {
            map.insert(HOSTNAME_VAR.into(), v.into());
        }
        Self(map)
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