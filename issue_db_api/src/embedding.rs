use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use serde_json::Value;
use crate::api_core::IssueAPI;
use crate::errors::APIResult;
use crate::config::ConfigHandlingPolicy;

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub(crate) struct UnboundEmbedding {
    pub(crate) embedding_id: String,
    pub(crate) name: String,
    pub(crate) config: HashMap<String, Value>,
    pub(crate) has_file: bool
}

impl UnboundEmbedding {
    pub(crate) fn into_bound_embedding(self,
                                       api: Arc<IssueAPI>,
                                       config_handling: ConfigHandlingPolicy) -> Embedding {
        Embedding{
            api,
            name: self.name,
            id: self.embedding_id,
            config: self.config,
            has_file: self.has_file,
            update_policy: config_handling
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Embedding {
    api: Arc<IssueAPI>,
    id: String,
    name: String ,
    config: HashMap<String, Value>,
    has_file: bool,
    update_policy: ConfigHandlingPolicy
}

impl PartialEq for Embedding {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Embedding {}

impl Hash for Embedding {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl Embedding {
    pub(crate) fn new(api: Arc<IssueAPI>,
                      id: String,
                      name: String,
                      config: HashMap<String, Value>,
                      has_file: bool,
                      update_policy: ConfigHandlingPolicy) -> Self {
        Embedding{api, id, name, config, has_file, update_policy}
    }

    pub fn identifier(&self) -> String {
        self.id.clone()
    }

    pub fn name(&self) -> APIResult<String> {
        match self.update_policy {
            ConfigHandlingPolicy::ReadFetchWriteWithFetch => {
                let name = self.api.get_embedding(self.id.clone())?.name;
                Ok(name)
            },
            _ => Ok(self.name.clone())
        }
    }

    pub fn update_name(&mut self, name: String) -> APIResult<()> {
        let config = match self.update_policy {
            ConfigHandlingPolicy::ReadLocalWriteNoFetch => self.config.clone(),
            _ => self.api.get_embedding(self.id.clone())?.config
        };
        self.api.update_embedding(self.id.clone(),
                                  name.clone(),
                                  config)?;
        self.name = name;
        Ok(())
    }

    pub fn config(&self) -> APIResult<HashMap<String, Value>> {
        match self.update_policy {
            ConfigHandlingPolicy::ReadFetchWriteWithFetch => {
                let config = self.api.get_embedding(self.id.clone())?.config;
                Ok(config)
            },
            _ => Ok(self.config.clone())
        }
    }

    pub fn update_config(&mut self, config: HashMap<String, Value>) -> APIResult<()> {
        let name = match self.update_policy {
            ConfigHandlingPolicy::ReadLocalWriteNoFetch => self.name.clone(),
            _ => self.api.get_embedding(self.id.clone())?.name
        };
        self.api.update_embedding(self.id.clone(),
                                  name,
                                  config.clone())?;
        self.config = config;
        Ok(())
    }

    pub fn download_binary(&self, path: String) -> APIResult<()> {
        self.api.download_embedding_binary(self.id.clone(), path)
    }

    pub fn upload_binary(&self, path: String) -> APIResult<()> {
        self.api.upload_embedding_binary(self.id.clone(), path)
    }

    pub fn delete_binary(&self) -> APIResult<()> {
        self.api.delete_embedding_binary(self.id.clone())
    }
}