use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;

use crate::api_core::IssueAPI;
use crate::config::{CachingPolicy, IssueLoadingSettings, ConfigHandlingPolicy};
use crate::embedding::Embedding;
use crate::issues::Issue;
use crate::models::Model;
use crate::query::Query;
use crate::tags::Tag;
use crate::errors::APIResult;


#[allow(unused)]
#[derive(Debug)]
pub struct IssueRepository {
    label_caching: CachingPolicy,
    config_handling: ConfigHandlingPolicy,
    api: Arc<IssueAPI>
}


#[allow(unused)]
impl IssueRepository {
    pub fn new_read_only(url: String,
                         label_caching_policy: CachingPolicy,
                         config_handling_policy: ConfigHandlingPolicy,
                         allow_self_signed_certs: bool) -> APIResult<Self> {
        Ok(
            Self{
                api: Arc::new(IssueAPI::new_read_only(url, allow_self_signed_certs)?),
                label_caching: label_caching_policy,
                config_handling: config_handling_policy
            }
        )
    }

    pub fn new(url: String,
               username: String,
               password: String,
               label_caching_policy: CachingPolicy,
               config_handling_policy: ConfigHandlingPolicy,
               allow_self_signed_certs: bool) -> APIResult<Self> {
        let api = Arc::new(IssueAPI::new(url, username, password, allow_self_signed_certs)?);
        Ok(
            Self{
                api,
                label_caching: label_caching_policy,
                config_handling: config_handling_policy
            }
        )
    }

    pub fn search(&self,
                  query: Query,
                  issue_loading_settings: IssueLoadingSettings) -> APIResult<Vec<Issue>> {
        let ids = self.api.search(query)?;
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        issue_loading_settings.load_issues(self.api.clone(),
                                           ids,
                                           self.label_caching)
    }

    pub fn find_issue_by_key(&self,
                             project: String,
                             name: String,
                             loading: IssueLoadingSettings) -> APIResult<Issue> {
        let id = self.api.find_issue_id_by_key(project, name)?;
        loading.load_issue(self.api.clone(), id, self.label_caching)
    }

    pub fn find_issues_by_key(&self,
                              issues: Vec<(String, String)>,
                              loading: IssueLoadingSettings) -> APIResult<Vec<Issue>> {
        let ids = self.api.find_issue_ids_by_keys(issues)?;
        loading.load_issues(self.api.clone(), ids, self.label_caching)
    }

    pub fn repos(&self) -> APIResult<Vec<IssueRepo>> {
        let repos = self.api.get_all_repos()?
            .into_iter()
            .map(|name| IssueRepo{name, api: self.api.clone()})
            .collect();
        Ok(repos)
    }

    pub fn tags(&self) -> APIResult<Vec<Tag>> {
        let tags = self.api.get_all_tags()?
            .into_iter()
            .map(|t| t.into_bound_tag(self.api.clone()))
            .collect();
        Ok(tags)
    }

    pub fn add_new_tag(&self, name: String, description: String) -> APIResult<()> {
        self.api.register_new_tag(name, description)
    }

    pub fn bulk_add_tags(&self, tags: HashMap<&Issue, Vec<String>>) -> APIResult<()> {
        let payload = tags.into_iter()
            .map(|(k, v)| (k.ident().clone(), v))
            .collect();
        self.api.bulk_add_tags(payload)
    }

    pub fn embeddings(&self) -> APIResult<Vec<Embedding>> {
        let embeddings = self.api.get_all_embeddings()?
            .into_iter()
            .map(|e| e.into_bound_embedding(self.api.clone(), self.config_handling))
            .collect();
        Ok(embeddings)
    }
    
    pub fn get_embedding_by_id(&self, id: String) -> APIResult<Embedding> {
        let raw = self.api.get_embedding(id)?;
        let converted = raw.into_bound_embedding(self.api.clone(), self.config_handling);
        Ok(converted)
    }

    pub fn create_embedding(&self,
                            name: String,
                            config: HashMap<String, Value>) -> APIResult<Embedding> {
        let id = self.api.create_embedding(name.clone(), config.clone())?;
        let embedding = Embedding::new(
            self.api.clone(),
            id,
            name,
            config,
            false,
            self.config_handling
        );
        Ok(embedding)
    }

    pub fn delete_embedding(&self, embedding: Embedding) -> APIResult<()> {
        self.api.delete_embedding(embedding.identifier())
    }

    pub fn models(&self) -> APIResult<Vec<Model>> {
        let models = self.api.get_all_models()?;
        let converted = models
            .into_iter()
            .map(|m|
                Model::new(self.api.clone(),
                           m.model_id,
                           m.model_name,
                           None,
                           self.config_handling)
            ).collect();
        Ok(converted)
    }
    
    pub fn get_model_by_id(&self, id: String) -> APIResult<Model> {
        let raw = self.api.get_model_config(id)?;
        let model = Model::new(
            self.api.clone(),
            raw.model_id,
            raw.model_name,
            Some(raw.model_config),
            self.config_handling
        );
        Ok(model)
    }

    pub fn add_model(&self, name: String, config: HashMap<String, Value>) -> APIResult<Model> {
        let id = self.api.create_model_config(name.clone(), config.clone())?;
        let m = Model::new(
            self.api.clone(),
            id,
            name,
            Some(config),
            self.config_handling
        );
        Ok(m)
    }

    pub fn delete_model_config(&self, model: Model) -> APIResult<()> {
        self.api.delete_model_config(model.identifier())
    }

    pub(crate) fn delete_model_config_by_id(&self, id: String) -> APIResult<()> {
        self.api.delete_model_config(id)
    }
}


pub struct IssueRepo {
    name: String,
    api: Arc<IssueAPI>
}

impl IssueRepo {
    pub fn name(&self) -> &String {
        &self.name 
    }
    
    pub fn projects(&self) -> APIResult<Vec<String>> {
        self.api.get_projects_for_repo(self.name.clone())
    }
}
