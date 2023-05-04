use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use crate::api_core::IssueAPI;
use crate::{Query, QueryCMP};
use crate::errors::{APIError, APIResult};
use crate::issues::Issue;

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ConfigHandlingPolicy {
    ReadLocalWriteNoFetch,
    ReadLocalWriteWithFetch,
    ReadFetchWriteWithFetch
}


#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CachingPolicy {
    NoCaching,
    UseLocalAfterLoad
}


#[allow(unused)]
#[derive(Debug, Copy, Clone)]
pub enum IssueAttribute {
    Key,
    Summary, Description, Comments,
    Parent, Subtasks, IssueLinks,
    Status, Priority, Resolution, IssueType,
    Watches, Votes,
    DateCreated, DateUpdated, DateResolved,
    Labels, Components,
    AffectedVersions, FixVersions
}

impl std::fmt::Display for IssueAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            IssueAttribute::Key => "key",
            IssueAttribute::Summary => "summary",
            IssueAttribute::Description => "description",
            IssueAttribute::Comments => "comments",
            IssueAttribute::Parent => "parent",
            IssueAttribute::Subtasks => "subtasks",
            IssueAttribute::IssueLinks => "issuelinks",
            IssueAttribute::Status => "status",
            IssueAttribute::Priority => "priority",
            IssueAttribute::Resolution => "resolution",
            IssueAttribute::IssueType => "issuetype",
            IssueAttribute::Watches => "watches",
            IssueAttribute::Votes => "votes",
            IssueAttribute::DateCreated => "created",
            IssueAttribute::DateUpdated => "updated",
            IssueAttribute::DateResolved => "resolutiondate",
            IssueAttribute::Labels => "labels",
            IssueAttribute::Components => "components",
            IssueAttribute::AffectedVersions => "versions",
            IssueAttribute::FixVersions => "fixVersions"
        };
        write!(f, "{}", text)
    }
}

impl TryFrom<String> for IssueAttribute {
    type Error = APIError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let attr = match value.as_str() {
            "key" => IssueAttribute::Key,
            "summary" => IssueAttribute::Summary,
            "description" => IssueAttribute::Description,
            "comments" => IssueAttribute::Comments,
            "parent" => IssueAttribute::Parent,
            "subtasks" => IssueAttribute::Subtasks,
            "issue_links" => IssueAttribute::IssueLinks,
            "status" => IssueAttribute::Status,
            "priority" => IssueAttribute::Priority,
            "resolution" => IssueAttribute::Resolution,
            "issue_type" => IssueAttribute::IssueType,
            "watches" => IssueAttribute::Watches,
            "votes" => IssueAttribute::Votes,
            "date_created" => IssueAttribute::DateCreated,
            "date_updated" => IssueAttribute::DateUpdated,
            "date_resolved" => IssueAttribute::DateResolved,
            "labels" => IssueAttribute::Labels,
            "components" => IssueAttribute::Components,
            "affected+versions" => IssueAttribute::AffectedVersions,
            "fix_versions" => IssueAttribute::FixVersions,
            _ => {
                let msg = format!("\"{}\" is an invalid issue attribute", value);
                return Err(APIError::GenericError(msg));
            }
        };
        Ok(attr)
    }
}


#[allow(unused)]
#[derive(Debug, Clone, Default)]
pub struct IssueLoadingSettings {
    attributes: Vec<IssueAttribute>,
    preload_labels: bool,
}


#[allow(unused)]
impl IssueLoadingSettings {
    pub fn new(attributes: Vec<IssueAttribute>,
               preload_labels: bool) -> Self {
        Self{attributes, preload_labels}
    }

    pub fn load_issues(self,
                       api: Arc<IssueAPI>,
                       ids: Vec<String>,
                       label_caching_policy: CachingPolicy) -> APIResult<Vec<Issue>> {
        let labels = if self.preload_labels {
            let query = Query::And(vec![
                Query::Tag(QueryCMP::Eq, "has-label".to_string()),
                Query::Or(ids.iter().cloned().map(Query::Identifier).collect())
            ]);
            let ids_with_label = api.search(query)?;
            api.get_manual_labels(ids_with_label)?
        } else {
            HashMap::new()
        };
        let data = api.get_issue_data(
            ids.clone(), self.attributes
        )?;
        let issues = ids.into_iter()
            .map(|id| Issue::new(
                api.clone(),
                id.clone(),
                data.get(&id).unwrap().clone(),
                label_caching_policy,
                labels.get(&id).cloned()))
            .collect();
        Ok(issues)
    }

    pub fn load_issue(self,
                      api: Arc<IssueAPI>,
                      id: String,
                      label_caching_policy: CachingPolicy) -> APIResult<Issue> {
        let issue = self.load_issues(api, vec![id], label_caching_policy)?
            .into_iter()
            .last()
            .expect("Failed to load issue");
        Ok(issue)
    }
}

