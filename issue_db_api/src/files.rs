use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::api_core::IssueAPI;
use crate::APIResult;

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub(crate) struct UnboundFile {
    pub(crate) file_id: String,
    description: String,
    category: String
}

impl UnboundFile {
    pub(crate) fn into_bound_file(self, api: Arc<IssueAPI>) -> File {
        File{
            api,
            id: self.file_id,
            description: self.description,
            category: self.category
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct File {
    api: Arc<IssueAPI>,
    id: String,
    description: String,
    category: String
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for File {}

impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

impl File {
    pub fn upload(api: Arc<IssueAPI>, path: String, description: String, category: String) -> APIResult<Self> {
        let id = api.upload_file(path, description.clone(), category.clone())?;
        let file = Self{api, id, description, category};
        Ok(file)
    }

    pub fn identifier(&self) -> String {
        self.id.clone()
    }

    pub fn description(&self) -> String {
        self.description.clone()
    }

    pub fn category(&self) -> String {
        self.category.clone()
    }

    pub fn download(&self, path: String) -> APIResult<()> {
        self.api.download_file(self.id.clone(), path)
    }

    pub fn delete(self) -> APIResult<()> {
        self.api.delete_file(self.id)
    }
}