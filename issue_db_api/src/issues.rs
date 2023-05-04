use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::api_core::{IssueAPI, IssueData};
use crate::comments::Comment;
use crate::config::{CachingPolicy, IssueLoadingSettings};
use crate::errors::APIResult;
use crate::labels::Label;
use crate::util::CacheContainer;

#[allow(unused)]
#[derive(Debug)]
pub struct Issue {
    api: Arc<IssueAPI>,
    ident: String,
    data: IssueData,
    caching_policy: CachingPolicy,
    label: CacheContainer<Option<Label>>,
    dirty: AtomicBool
}

impl Hash for Issue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ident.hash(state);
    }
}

impl PartialEq for Issue {
    fn eq(&self, other: &Self) -> bool {
        self.ident == other.ident
    }
}

impl Eq for Issue {}

#[allow(unused)]
impl Issue {
    pub(crate) fn new(api: Arc<IssueAPI>,
                      ident: String,
                      data: IssueData,
                      caching: CachingPolicy,
                      label: Option<Label>) -> Self {
         Self{
             api, ident, data,
             caching_policy: caching,
             label: CacheContainer::new(Some(label)),
             dirty: AtomicBool::new(false)
        }
    }

    #[inline(always)]
    pub fn ident(&self) -> &String {
        &self.ident
    }

    pub fn get_manual_label(&self) -> APIResult<Option<Label>> {
        self.label.get(|| self.load_label())
    }

    fn load_label(&self) -> APIResult<Option<Label>> {
        let labels = self.api.get_manual_labels(
            vec![self.ident.clone()]
        )?;
        Ok(labels.get(&self.ident).copied())
    }

    pub fn set_manual_label(&self, label: Label) -> APIResult<()> {
        self.label.set(Some(label))
    }

    pub fn invalidate_cached_label(&self) {
        self.dirty.store(true, Ordering::Release);
    }

    pub fn get_tags(&self) -> APIResult<Vec<String>> {
        self.api.get_tags_for_issue(self.ident.clone())
    }

    pub fn add_tag(&self, name: String) -> APIResult<()> {
        self.api.add_tag_to_issue(self.ident.clone(), name)
    }

    pub fn remove_tag(&self, name: String) -> APIResult<()> {
        self.api.remove_tag_from_issue(self.ident.clone(), name)
    }

    pub fn start_review(&self) -> APIResult<()> {
        self.api.start_issue_review(self.ident.clone())
    }

    pub fn finish_review(&self) -> APIResult<()> {
        self.api.finish_issue_review(self.ident.clone())
    }

    pub fn in_review(&self) -> APIResult<bool> {
        Ok(self.get_tags()?.contains(&"needs-review".to_string()))
    }

    pub fn get_labelling_comments(&self) -> APIResult<Vec<Comment>> {
        let comments = self.api
            .get_labeling_comments_for_issue(self.ident.clone())?
            .into_iter()
            .map(|r| r.into_comment(self.api.clone(), self.ident.clone()))
            .collect();
        Ok(comments)
    }

    pub fn add_labelling_comment(&self, text: String) -> APIResult<()> {
        let _ = self.api.add_labeling_comment_to_issue(self.ident.clone(), text)?;
        Ok(())
    }

    pub fn remove_labelling_comment(&self, comment: Comment) -> APIResult<()> {
        self.api.delete_labeling_comment(self.ident.clone(), comment.identifier().clone())
    }

    #[inline(always)]
    pub fn key(&self) -> APIResult<&String> {
        self.data.key(&self.api)
    }

    #[inline(always)]
    pub fn summary(&self) -> APIResult<&String> {
        self.data.summary(&self.api)
    }

    #[inline(always)]
    pub fn description(&self) -> APIResult<&String> {
        self.data.description(&self.api)
    }

    #[inline(always)]
    pub fn comments(&self) -> APIResult<&Vec<String>> {
        self.data.comments(&self.api)
    }

    #[inline(always)]
    pub fn status(&self) -> APIResult<&String> {
        self.data.status(&self.api)
    }

    #[inline(always)]
    pub fn priority(&self) -> APIResult<&String> {
        self.data.priority(&self.api)
    }

    #[inline(always)]
    pub fn resolution(&self) -> APIResult<&Option<String>> {
        self.data.resolution(&self.api)
    }

    #[inline(always)]
    pub fn issue_type(&self) -> APIResult<&String> {
        self.data.issue_type(&self.api)
    }

    #[inline(always)]
    pub fn issue_links(&self, loading_settings: IssueLoadingSettings) -> APIResult<Vec<Issue>> {
        let ids = self.data.issue_links(&self.api)?;
        let issues = loading_settings.load_issues(
            self.api.clone(), ids.clone(), self.caching_policy
        )?;
        assert_eq!(issues.len(), ids.len(), "Failed to retrieve all linked issues");
        Ok(issues)
    }

    #[inline(always)]
    pub fn parent(&self, loading_settings: IssueLoadingSettings) -> APIResult<Option<Issue>> {
        if let Some(id) = self.data.parent(&self.api)? {
            Ok(
                Some(
                    loading_settings.load_issue(self.api.clone(),
                                                id.clone(),
                                                self.caching_policy)?
                )
            )
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    pub fn subtasks(&self, loading_settings: IssueLoadingSettings) -> APIResult<Vec<Issue>> {
        let ids = self.data.subtasks(&self.api)?;
        let issues = loading_settings.load_issues(
            self.api.clone(), ids.clone(), self.caching_policy
        )?;
        assert_eq!(issues.len(), ids.len(), "Failed to retrieve all subtasks");
        Ok(issues)
    }

    #[inline(always)]
    pub fn watches(&self) -> APIResult<u64> {
        self.data.watches(&self.api)
    }

    #[inline(always)]
    pub fn votes(&self) -> APIResult<u64> {
        self.data.votes(&self.api)
    }

    #[inline(always)]
    pub fn date_created(&self) -> APIResult<&String> {
        self.data.date_created(&self.api)
    }

    #[inline(always)]
    pub fn date_updated(&self) -> APIResult<&String> {
        self.data.date_updated(&self.api)
    }

    #[inline(always)]
    pub fn date_resolved(&self) -> APIResult<&Option<String>> {
        self.data.date_resolved(&self.api)
    }

    #[inline(always)]
    pub fn labels(&self) -> APIResult<&Vec<String>> {
        self.data.labels(&self.api)
    }

    #[inline(always)]
    pub fn components(&self) -> APIResult<&Vec<String>> {
        self.data.components(&self.api)
    }

    #[inline(always)]
    pub fn affected_versions(&self) -> APIResult<&Vec<String>> {
        self.data.affected_versions(&self.api)
    }

    #[inline(always)]
    pub fn fix_versions(&self) -> APIResult<&Vec<String>> {
        self.data.fix_versions(&self.api)
    }
}
