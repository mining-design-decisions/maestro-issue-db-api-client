use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use serde_json::Value;
use crate::api_core::IssueAPI;
use crate::config::ConfigHandlingPolicy;
use crate::errors::{APIError, APIResult};

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub(crate) struct UnboundProject {
    pub(crate) ecosystem: String,
    pub(crate) key: String,
    #[serde(rename(deserialize = "additional_properties"))] pub(crate) properties: HashMap<String, Value>
}

impl UnboundProject {
    pub(crate) fn into_bound_project(self, api: Arc<IssueAPI>,
                                     update_policy: ConfigHandlingPolicy) -> Project {
        Project {
            api,
            ecosystem: self.ecosystem,
            key: self.key,
            properties: self.properties,
            update_policy
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Project {
    api: Arc<IssueAPI>,
    ecosystem: String,
    key: String,
    properties: HashMap<String, Value>,
    update_policy: ConfigHandlingPolicy
}

impl PartialEq for Project {
    fn eq(&self, other: &Self) -> bool {
        self.ecosystem == other.ecosystem && self.key == other.key
    }
}

impl Eq for Project {}

impl Hash for Project {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ecosystem.hash(state);
        self.key.hash(state);
    }
}

impl Project {
    pub(crate) fn new(api: Arc<IssueAPI>,
                      ecosystem: String,
                      key: String,
                      properties: HashMap<String, Value>,
                      update_policy: ConfigHandlingPolicy) -> Self {
        Self{api, ecosystem, key, properties, update_policy}
    }

    pub fn ecosystem(&self) -> String {
        self.ecosystem.clone()
    }

    pub fn key(&self) -> String {
        self.key.clone()
    }

    pub fn properties(&self) -> APIResult<HashMap<String, Value>> {
        match self.update_policy {
            ConfigHandlingPolicy::ReadFetchWriteWithFetch => {
                let result = self.api
                    .get_project(self.ecosystem.clone(), self.key.clone())?
                    .properties;
                Ok(result)
            },
            _ => Ok(self.properties.clone())
        }
    }

    pub fn get_property(&self, name: String) -> APIResult<Value> {
        // self.properties
        //     .get(&name)
        //     .ok_or(APIError::GenericError(format!("Unknown property: {name}")))
        let properties = self.properties()?;
        let value = properties
            .get(&name)
            .ok_or(APIError::GenericError(format!("Unknown property: {name}")))
            .map(|x| x.clone());
        value
    }

    pub fn set_property(&mut self, name: String, value: Value) -> APIResult<()> {
        let mut base_payload = match self.update_policy {
            ConfigHandlingPolicy::ReadLocalWriteNoFetch => self.properties.clone(),
            _ => {
               self.api
                   .get_project(self.ecosystem.clone(), self.key.clone())?
                   .properties
            }
        };
        base_payload.insert(name, value);
        self.set_properties(base_payload)
    }

    pub fn set_properties(&mut self, properties: HashMap<String, Value>) -> APIResult<()> {
        self.properties = properties.clone();
        self.api.update_project_properties(self.ecosystem.clone(),
                                           self.key.clone(),
                                           properties)
    }
}