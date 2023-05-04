use std::sync::Arc;
use crate::api_core::IssueAPI;
use crate::errors::APIResult;

#[allow(unused)]
#[derive(Debug)]
pub struct UnboundComment {
    pub(crate) id: String,
    pub(crate) author: String,
    pub(crate) text: String
}

impl UnboundComment {
    pub fn into_comment(self, api: Arc<IssueAPI>, parent: String) -> Comment {
        Comment{api, parent, id: self.id, author: self.author, text: self.text}
    }
}


#[allow(unused)]
#[derive(Debug, Clone)]
pub struct Comment {
    api: Arc<IssueAPI>,
    parent: String,
    id: String,
    author: String,
    text: String
}

impl Comment {
    pub(crate) fn identifier(&self) -> &String {
        &self.id
    }

    pub fn author(&self) -> &String {
        &self.author
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn update_text(&mut self, text: String) -> APIResult<()> {
        self.api.update_labeling_comment(
            self.parent.clone(),
            self.id.clone(),
            text.clone())?;
        self.text = text;
        Ok(())
    }
}