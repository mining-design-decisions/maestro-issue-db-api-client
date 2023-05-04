use std::hash::{Hash, Hasher};
use std::sync::Arc;
use crate::api_core::IssueAPI;
use crate::errors::APIResult;

#[allow(unused)]
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub enum TagType {
    #[serde(rename="author")] Author,
    #[serde(rename="project")] Project,
    #[serde(rename="manual-tag")] Custom
}

#[allow(unused)]
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct UnboundTag {
    pub(crate) name: String,
    pub(crate) description: String,
    #[serde(rename(deserialize="type"))] pub(crate) tag_type: TagType
}

impl UnboundTag {
    pub(crate) fn into_bound_tag(self, api: Arc<IssueAPI>) -> Tag {
        Tag{
            api,
            name: self.name,
            description: self.description,
            tag_type: self.tag_type
        }
    }
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Tag {
    api: Arc<IssueAPI>,
    name: String,
    description: String,
    tag_type: TagType
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Tag {}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

#[allow(unused)]
impl Tag {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn description(&self) -> &String {
        &self.description
    }

    pub fn update_description(&mut self, description: String) -> APIResult<()> {
        self.api.update_tag(self.name.clone(), description.clone())?;
        self.description = description;
        Ok(())
    }

    pub fn tag_type(&self) -> TagType {
        self.tag_type
    }
}
