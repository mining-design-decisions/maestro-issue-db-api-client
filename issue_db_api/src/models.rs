use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use serde_json::Value;
use crate::api_core::IssueAPI;
use crate::APIResult;
use crate::config::ConfigHandlingPolicy;
use crate::util::CacheContainer;

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Raw Models -- Used for Raw API responses
//////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub(crate) struct ModelInfo {
    pub(crate) model_id: String,
    pub(crate) model_name: String,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub(crate) struct UnboundModelConfig {
    pub(crate) model_id: String,
    pub(crate) model_name: String,
    pub(crate) model_config: HashMap<String, Value>,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub(crate) struct UnboundModelVersion {
    pub(crate) model_id: String,
    pub(crate) version_id: String,
    pub(crate) time: String,
}

impl UnboundModelVersion {
    fn into_bound_model(self, api: Arc<IssueAPI>) -> ModelVersion {
        ModelVersion{
            api,
            model: self.model_id,
            version: self.version_id,
            time: self.time
        }
    }
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub(crate) struct UnboundTestRun {
    pub(crate) model_id: String,
    pub(crate) performance_id: String
}

impl UnboundTestRun {
    fn into_bound_test_run(self, api: Arc<IssueAPI>) -> TestRun {
        TestRun{api, model: self.model_id, run: self.performance_id}
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Higher level "OOP" models
//////////////////////////////////////////////////////////////////////////////////////////////////


#[allow(unused)]
#[derive(Debug)]
pub struct Model {
    api: Arc<IssueAPI>,
    id: String,
    name: String,
    config: CacheContainer<HashMap<String, Value>>,
    data_policy: ConfigHandlingPolicy
}

impl PartialEq for Model {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Model {}

impl Hash for Model {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

#[allow(unused)]
impl Model {
    pub fn new(api: Arc<IssueAPI>,
               id: String,
               name: String,
               config: Option<HashMap<String, Value>>,
               config_handling: ConfigHandlingPolicy) -> Self {
        Self{
            api,
            id,
            name,
            config: CacheContainer::new(config),
            data_policy: config_handling
        }
    }

    pub(crate) fn identifier(&self) -> &String {
        &self.id
    }

    pub fn name(&self) -> APIResult<String> {
        match self.data_policy {
            ConfigHandlingPolicy::ReadFetchWriteWithFetch => {
                Ok(self.api.get_model_config(self.id.clone())?.model_name)
            },
            _ => Ok(self.name.clone())
        }
    }

    pub fn update_name(&mut self, name: String) -> APIResult<()> {
        let config = match self.data_policy {
            ConfigHandlingPolicy::ReadLocalWriteNoFetch => {
                self.config.get(
                    || Ok(self.api.get_model_config(self.id.clone())?.model_config)
                )?
            },
            _ => {
                self.api.get_model_config(self.id.clone())?.model_config
            }
        };
        self.api.update_model_config(
            self.id.clone(),
            name.clone(),
            config
        );
        self.name = name;
        Ok(())
    }

    pub fn config(&self) -> APIResult<HashMap<String, Value>> {
        match self.data_policy {
            ConfigHandlingPolicy::ReadFetchWriteWithFetch => {
                Ok(self.api.get_model_config(self.id.clone())?.model_config)
            },
            _ => self.config.get(
                || Ok(self.api.get_model_config(self.id.clone())?.model_config)
            )
        }
    }

    pub fn update_config(&mut self, config: HashMap<String, Value>) -> APIResult<()> {
        let name = match self.data_policy {
            ConfigHandlingPolicy::ReadLocalWriteNoFetch => self.name.clone(),
            _ => {
                self.api.get_model_config(self.id.clone())?.model_name
            }
        };
        self.api.update_model_config(self.id.clone(), name , config.clone());
        self.config.set(config)?;
        Ok(())
    }

    pub fn upload_version(&self, time: String, path: String) -> APIResult<ModelVersion> {
        let id = self.api.upload_model_version(
            self.id.clone(), time.clone(), path
        )?;
        let v = ModelVersion{api: self.api.clone(), model: self.id.clone(), version: id, time};
        Ok(v)
    }

    pub fn delete_version(&self, model: ModelVersion) -> APIResult<()> {
        self.api.delete_model_version(
            self.id.clone(), model.version_id().clone()
        )
    }

    pub fn model_versions(&self) -> APIResult<Vec<ModelVersion>> {
        let versions = self.api.get_versions_for_model(self.id.clone())?;
        let converted = versions
            .into_iter()
            .map(|v| v.into_bound_model(self.api.clone()))
            .collect();
        Ok(converted)
    }

    pub fn model_runs(&self) -> APIResult<Vec<TestRun>> {
        let runs = self.api.get_performances_for_model(self.id.clone())?;
        let converted = runs
            .into_iter()
            .map(|r| r.into_bound_test_run(self.api.clone()))
            .collect();
        Ok(converted)
    }

    pub fn store_run(&self, data: Vec<Value>) -> APIResult<TestRun> {
        let id = self.api.store_model_performance(self.id.clone(), data)?;
        let run = TestRun{api: self.api.clone(), model: self.id.clone(), run: id};
        Ok(run)
    }

    pub fn delete_run(&self, run: TestRun) -> APIResult<()> {
        self.api.delete_performance_data(run.model, run.run)
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct ModelVersion {
    api: Arc<IssueAPI>,
    model: String,
    version: String,
    time: String
}

impl PartialEq for ModelVersion {
    fn eq(&self, other: &Self) -> bool {
        self.model == other.model && self.version == other.version
    }
}

impl Eq for ModelVersion {}

impl Hash for ModelVersion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.version.hash(state);
        self.model.hash(state);
    }
}

#[allow(unused)]
impl ModelVersion {
    pub(crate) fn model_id(&self) -> &String {
        &self.model
    }

    pub(crate) fn version_id(&self) -> &String {
        &self.version
    }

    pub fn time(&self) -> &String {
        &self.time
    }

    pub fn download(&self, path: String) -> APIResult<()> {
        self.api.download_model_version(self.model.clone(),
                                        self.version.clone(),
                                        path)
    }

    pub fn get_predictions(&self, issues: Option<Vec<String>>) -> APIResult<HashMap<String, Value>> {
        self.api.get_predictions(self.model.clone(), self.version.clone(), issues)
    }

    pub fn store_predictions(&self, predictions: HashMap<String, Value>) -> APIResult<()> {
        self.api.store_predictions(self.model.clone(), self.version.clone(), predictions)
    }

    pub fn delete_predictions(&self) -> APIResult<()> {
        self.api.delete_predictions(self.model.clone(), self.version.clone())
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct TestRun {
    api: Arc<IssueAPI>,
    model: String,
    run: String
}

#[allow(unused)]
impl TestRun {
    pub(crate) fn model_id(&self) -> &String {
        &self.model
    }

    pub(crate) fn run_id(&self) -> &String {
        &self.run
    }

    pub fn data(&self) -> APIResult<Vec<Value>> {
        self.api.get_performance_data(self.model.clone(), self.run.clone())
    }
}