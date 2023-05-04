use std::collections::HashMap;
use std::io::Cursor;

#[cfg(feature = "blocking")]
use reqwest::blocking::Client;
use reqwest::blocking::{ClientBuilder, multipart};

use serde_json::{Map, Value};
use lazy_init::Lazy;

use crate::comments::UnboundComment;
use crate::config::IssueAttribute;
use crate::embedding::UnboundEmbedding;
use crate::schemas::raw_issue_response::RawIssueData;
use crate::query::Query;
use crate::labels::Label;
use crate::tags::UnboundTag;
use crate::util::initialize_lazy_field;
use crate::errors::APIResult;
use crate::errors::*;
use crate::models::{ModelInfo, UnboundModelConfig, UnboundModelVersion, UnboundTestRun};

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Core Structs
//////////////////////////////////////////////////////////////////////////////////////////////////

/// The IssueAPI struct provides a low-level wrapper around the issues API.
/// This struct provides basics means of session management,
/// and exposes all available endpoints through Rust functions.
#[allow(unused)]
#[derive(Debug, Clone)]
pub struct IssueAPI {
    url: String,
    token: Option<String>,
    client: Client,
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub struct IssueData {
    ident: String,
    key: Lazy<String>,
    summary: Lazy<String>,
    description: Lazy<String>,
    comments: Lazy<Vec<String>>,
    status: Lazy<String>,
    priority: Lazy<String>,
    resolution: Lazy<Option<String>>,
    issue_type: Lazy<String>,
    issue_links: Lazy<Vec<String>>,
    parent: Lazy<Option<String>>,
    subtasks: Lazy<Vec<String>>,
    watches: Lazy<u64>,
    votes: Lazy<u64>,
    date_created: Lazy<String>,
    date_updated: Lazy<String>,
    date_resolved: Lazy<Option<String>>,
    labels: Lazy<Vec<String>>,
    components: Lazy<Vec<String>>,
    affected_versions: Lazy<Vec<String>>,
    fix_versions: Lazy<Vec<String>>
}

//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Auxiliary structs and enums
//////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
#[allow(unused)]
enum Verb {
    Get,
    Post,
    Patch,
    Put,
    Delete
}

macro_rules! maybe_copy_attribute {
    ($self:ident, $other:ident, $attr:ident) => {
        {
            if let Some(x) = $other.$attr.get() {
                $self.$attr.get_or_create(|| x.clone());
            }
        }
    }
}

macro_rules! load_lazy_attribute {
    ($self:ident, $attr:ident, $e:expr, $api:expr) => {
        {
            if let Some(x) = $self.$attr.get() {
                Ok(x)
            } else {
                let value = $api.get_issue_data(vec![$self.ident.clone()], vec![$e])?
                    .get(&$self.ident)
                    .expect("Issue data lookup failed")
                    .$attr
                    .get()
                    .expect("Invalid missing data during lookup")
                    .clone();
                let x = $self.$attr.get_or_create(|| value);
                Ok(x)
            }
        }
    }
}

#[allow(unused)]
impl IssueData {
    pub(crate) fn new_empty(ident: String) -> Self {
        Self::from_raw_data(ident, RawIssueData::default())
    }

    fn from_raw_data(ident: String, value: RawIssueData) -> Self {
        Self {
            ident,
            key: initialize_lazy_field!(value.key),
            summary: initialize_lazy_field!(value.summary),
            description: initialize_lazy_field!(value.description),
            comments: initialize_lazy_field!(
                value.comments.map(|v| v.into_iter().map(|c| c.body).collect())
            ),
            status: initialize_lazy_field!(value.status.map(|x| x.name)),
            resolution: initialize_lazy_field!(
                value.resolution.map(|x| x.map(|y| y.name))
            ),
            priority: initialize_lazy_field!(value.priority.map(|x| x.name)),
            issue_type: initialize_lazy_field!(value.issuetype.map(|x| x.name)),
            issue_links: initialize_lazy_field!(Some(vec![])),
            parent: initialize_lazy_field!(value.parent),
            subtasks: initialize_lazy_field!(value.subtasks),
            watches: initialize_lazy_field!(value.watches.map(|x| x.watch_count)),
            votes: initialize_lazy_field!(value.votes.map(|x| x.votes)),
            date_created: initialize_lazy_field!(value.created),
            date_updated: initialize_lazy_field!(value.updated),
            date_resolved: initialize_lazy_field!(value.resolutiondate),
            labels: initialize_lazy_field!(value.labels),
            components: initialize_lazy_field!(
                value.components.map(|v| v.into_iter().map(|c| c.name).collect())
            ),
            affected_versions: initialize_lazy_field!(
                value.versions.map(|v| v.into_iter().map(|c| c.name).collect())
            ),
            fix_versions: initialize_lazy_field!(
                value.fix_versions.map(|v| v.into_iter().map(|c| c.name).collect())
            )
        }
    }

    pub(crate) fn update(&self, other: IssueData) {
        maybe_copy_attribute!(self, other, key);
        maybe_copy_attribute!(self, other, summary);
        maybe_copy_attribute!(self, other, description);
        maybe_copy_attribute!(self, other, comments);
        maybe_copy_attribute!(self, other, parent);
        maybe_copy_attribute!(self, other, subtasks);
        maybe_copy_attribute!(self, other, issue_links);
        maybe_copy_attribute!(self, other, status);
        maybe_copy_attribute!(self, other, priority);
        maybe_copy_attribute!(self, other, resolution);
        maybe_copy_attribute!(self, other, issue_type);
        maybe_copy_attribute!(self, other, watches);
        maybe_copy_attribute!(self, other, votes);
        maybe_copy_attribute!(self, other, date_created);
        maybe_copy_attribute!(self, other, date_updated);
        maybe_copy_attribute!(self, other, date_resolved);
        maybe_copy_attribute!(self, other, labels);
        maybe_copy_attribute!(self, other, components);
        maybe_copy_attribute!(self, other, affected_versions);
        maybe_copy_attribute!(self, other, fix_versions);
    }

    pub fn key(&self, api: &IssueAPI) -> APIResult<&String> {
        load_lazy_attribute!(self, key, IssueAttribute::Key, api)
    }

    pub fn summary(&self, api: &IssueAPI) -> APIResult<&String> {
        load_lazy_attribute!(self, summary, IssueAttribute::Summary, api)
    }

    pub fn description(&self, api: &IssueAPI) -> APIResult<&String> {
        load_lazy_attribute!(self, description, IssueAttribute::Description, api)
    }

    pub fn comments(&self, api: &IssueAPI) -> APIResult<&Vec<String>> {
        load_lazy_attribute!(self, comments, IssueAttribute::Comments, api)
    }

    pub fn parent(&self, api: &IssueAPI) -> APIResult<&Option<String>> {
        load_lazy_attribute!(self, parent, IssueAttribute::Parent, api)
    }

    pub fn subtasks(&self, api: &IssueAPI) -> APIResult<&Vec<String>> {
        load_lazy_attribute!(self, subtasks, IssueAttribute::Subtasks, api)
    }

    pub fn issue_links(&self, api: &IssueAPI) -> APIResult<&Vec<String>> {
        load_lazy_attribute!(self, issue_links, IssueAttribute::IssueLinks, api)
    }

    pub fn status(&self, api: &IssueAPI) -> APIResult<&String> {
        load_lazy_attribute!(self, status, IssueAttribute::Status, api)
    }

    pub fn priority(&self, api: &IssueAPI) -> APIResult<&String> {
        load_lazy_attribute!(self, priority, IssueAttribute::Priority, api)
    }

    pub fn resolution(&self, api: &IssueAPI) -> APIResult<&Option<String>> {
        load_lazy_attribute!(self, resolution, IssueAttribute::Resolution, api)
    }

    pub fn issue_type(&self, api: &IssueAPI) -> APIResult<&String> {
        load_lazy_attribute!(self, issue_type, IssueAttribute::IssueType, api)
    }

    pub fn watches(&self, api: &IssueAPI) -> APIResult<u64> {
        load_lazy_attribute!(self, watches, IssueAttribute::Watches, api).map(|x| *x)
    }

    pub fn votes(&self, api: &IssueAPI) -> APIResult<u64> {
        load_lazy_attribute!(self, votes, IssueAttribute::Votes, api).map(|x| *x)
    }

    pub fn date_created(&self, api: &IssueAPI) -> APIResult<&String> {
        load_lazy_attribute!(self, date_created, IssueAttribute::DateCreated, api)
    }

    pub fn date_updated(&self, api: &IssueAPI) -> APIResult<&String> {
        load_lazy_attribute!(self, date_updated, IssueAttribute::DateUpdated, api)
    }

    pub fn date_resolved(&self, api: &IssueAPI) -> APIResult<&Option<String>> {
        load_lazy_attribute!(self, date_resolved, IssueAttribute::DateResolved, api)
    }

    pub fn labels(&self, api: &IssueAPI) -> APIResult<&Vec<String>> {
        load_lazy_attribute!(self, labels, IssueAttribute::Labels, api)
    }

    pub fn components(&self, api: &IssueAPI) -> APIResult<&Vec<String>> {
        load_lazy_attribute!(self, components, IssueAttribute::Components, api)
    }

    pub fn affected_versions(&self, api: &IssueAPI) -> APIResult<&Vec<String>> {
        load_lazy_attribute!(self, affected_versions, IssueAttribute::AffectedVersions, api)
    }

    pub fn fix_versions(&self, api: &IssueAPI) -> APIResult<&Vec<String>> {
        load_lazy_attribute!(self, fix_versions, IssueAttribute::FixVersions, api)
    }
}


//////////////////////////////////////////////////////////////////////////////////////////////////
//////////////////////////////////////////////////////////////////////////////////////////////////
// Blocking implementation
//////////////////////////////////////////////////////////////////////////////////////////////////

#[allow(unused)]
#[cfg(feature = "blocking")]
impl IssueAPI {
    pub(crate) fn new(url: String,
                      username: String,
                      password: String,
                      allow_self_signed: bool) -> APIResult<Self> {
        let mut read_only_api = Self::new_read_only(url, allow_self_signed)?;
        read_only_api.login(username, password)?;
        Ok(read_only_api)
    }

    pub(crate) fn new_read_only(url: String, allow_self_signed: bool) -> APIResult<Self> {
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(allow_self_signed)
            .build()?;
        Ok(IssueAPI{url, token: None, client})
    }

    fn login(&mut self, username: String, password: String) -> APIResult<()> {
        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
            #[allow(dead_code)] token_type: String
        }
        let mut form = HashMap::new();
        form.insert("username", username);
        form.insert("password", password);
        let token = self.client
            .post(self.get_endpoint("token"))
            .form(&form)
            .send()?
            .error_for_status()?
            .json::<TokenResponse>()
            .expect("Received invalid token payload from server")
            .access_token;
        self.token = Some(token);
        Ok(())
    }

    /***************************************************************************
     * Shared Networking Code
     */

    fn get_endpoint(&self, suffix: &str) -> String {
        self.url.clone() + "/" + suffix
    }

    fn get_auth(&self) -> APIResult<String> {
        if self.token.is_none() {
            Err(APIError::NotAuthorized)
        } else {
            Ok(self.token.clone().unwrap())
        }
    }

    fn build_request_base(&self, suffix: &str, verb: Verb) -> APIResult<reqwest::blocking::RequestBuilder> {
        let url = self.get_endpoint(suffix);
        let request_base = match verb {
            Verb::Get => self.client.get(url),
            Verb::Post => self.client.post(url).bearer_auth(self.get_auth()?),
            Verb::Patch => self.client.patch(url).bearer_auth(self.get_auth()?),
            Verb::Put => self.client.put(url).bearer_auth(self.get_auth()?),
            Verb::Delete => self.client.delete(url).bearer_auth(self.get_auth()?)
        };
        Ok(request_base)
    }

    fn handle_error_status(&self,
                           response: reqwest::blocking::Response) -> APIResult<reqwest::blocking::Response> {
        Ok(response.error_for_status()?)
    }

    fn unpack_response<O>(&self, response: reqwest::blocking::Response) -> APIResult<O>
    where
        O: for <'de> serde::Deserialize<'de>
    {
        let result = self.handle_error_status(response)?
            .json::<O>()
            .expect("Received invalid response from server.");
        Ok(result)
    }

    fn call_endpoint_json<I, O>(&self,
                                suffix: &str,
                                verb: Verb,
                                payload: I) -> APIResult<O>
    where
        I: serde::Serialize + std::fmt::Debug,
        O: for <'de> serde::Deserialize<'de>,
    {
        let response = self.build_request_base(suffix, verb)?
            .json(&payload)
            .send()?;
        let result = self.unpack_response(response)?;
        Ok(result)
    }

    fn call_endpoint_form<I, O>(&self,
                                suffix: &str,
                                verb: Verb,
                                payload: &I) -> APIResult<O>
        where
            I: serde::Serialize + ?Sized,
            O: for <'de> serde::Deserialize<'de>,
    {
        let response = self.build_request_base(suffix, verb)?
            .form(payload)
            .send()?;
        let result = self.unpack_response(response)?;
        Ok(result)
    }

    fn call_endpoint_multipart<O>(&self,
                                  suffix: &str,
                                  verb: Verb,
                                  payload: multipart::Form) -> APIResult<O>
        where
            O: for <'de> serde::Deserialize<'de>,
    {
        let response = self.build_request_base(suffix, verb)?
            .multipart(payload)
            .send()?;
        let result = self.unpack_response(response)?;
        Ok(result)
    }

    fn call_endpoint_multipart_object<I, O>(&self,
                                            suffix: &str,
                                            verb: Verb,
                                            payload: &I) -> APIResult<O>
        where
            I: serde::Serialize + ?Sized,
            O: for <'de> serde::Deserialize<'de>,
    {
        let mut buffer = Cursor::new(Vec::new());
        serde_json::to_writer(&mut buffer, payload)?;
        buffer.set_position(0);
        let part = multipart::Part::reader(buffer)
            .mime_str("application/json")?
            .file_name("file.json");
        let form = multipart::Form::new().part("file", part);
        self.call_endpoint_multipart(suffix, verb, form)
    }

    fn call_endpoint_download<I>(&self,
                                 suffix: &str,
                                 verb: Verb,
                                 payload: I,
                                 target_path: String) -> APIResult<()>
        where
            I: serde::Serialize,
    {
        let mut file = std::fs::File::create(target_path)?;
        self.call_endpoint_download_into(suffix, verb, payload, &mut file)
    }

    fn call_endpoint_download_object<I, O>(&self,
                                           suffix: &str,
                                           verb: Verb,
                                           payload: I) -> APIResult<O>
        where
            I: serde::Serialize,
            O: for <'de> serde::Deserialize<'de>
    {
        let mut buffer = std::io::Cursor::new(Vec::new());
        self.call_endpoint_download_into(suffix, verb, payload, &mut buffer)?;
        buffer.set_position(0);
        let result: O = serde_json::from_reader(buffer)?;
        Ok(result)
    }

    fn call_endpoint_download_into<I, S>(&self,
                                         suffix: &str,
                                         verb: Verb,
                                         payload: I,
                                         sink: &mut S) -> APIResult<()>
        where
            I: serde::Serialize,
            S: std::io::Write
    {
        let url = self.get_endpoint(suffix);
        let client = reqwest::Client::new();
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
        let result: APIResult<()> = rt.block_on(async {
            let mut stream = client
                .get(url)
                .send()
                .await?
                .error_for_status()?;
            while let Some(chunk) = stream.chunk().await? {
                sink.write_all(chunk.as_ref())?;
            }
            Ok(())
        });
        result
    }

    /***************************************************************************
     * Issue-related endpoints
     */

    pub(crate) fn search(&self, query: Query) -> APIResult<Vec<String>> {
        let mut map = Map::new();
        map.insert("filter".to_string(), query.into_json());
        #[derive(serde::Deserialize)]
        struct SearchResponse {
            issue_ids: Vec<String>
        }
        let result = self.call_endpoint_json::<_, SearchResponse>(
            "issue-ids",
            Verb::Get,
            Value::Object(map)
        )?;
        Ok(result.issue_ids)
    }

    pub(crate) fn find_issue_id_by_key(&self, project: String, key: String) -> APIResult<String> {
        #[derive(serde::Deserialize)]
        struct IdResponse {
            issue_id: String
        }
        let endpoint = format!("issue-ids/{}/{}", project, key);
        let result = self.call_endpoint_json::<_, IdResponse>(
            endpoint.as_str(),
            Verb::Get,
            Value::Object(Map::new())
        )?;
        Ok(result.issue_id)
    }

    pub(crate) fn find_issue_ids_by_keys(&self, ids: Vec<(String, String)>) -> APIResult<Vec<String>> {
        #[derive(serde::Deserialize)]
        struct IdsResponse {
            issue_ids: HashMap<String, String>
        }
        let keys = ids.into_iter()
            .map(|(project, key)| format!("{}-{}", project, key))
            .collect::<Vec<_>>();
        let mut map = Map::new();
        map.insert(
            "issue_keys".to_string(),
            Value::Array(keys.iter().cloned().map(Value::String).collect())
        );
        let result = self.call_endpoint_json::<_, IdsResponse>(
            "bulk/get-issue-ids-from-keys",
            Verb::Get,
            Value::Object(map)
        )?;
        let mut ids: Vec<String> = Vec::with_capacity(keys.len());
        for key in keys {
            match result.issue_ids.get(&key) {
                None => {
                    let msg = format!("No ID found for key \"{}\"", key);
                    return Err(APIError::GenericError(msg));
                },
                Some(id) => ids.push(id.clone())
            }
        }
        Ok(ids)
    }

    pub(crate) fn get_issue_data(&self,
                          issues: Vec<String>,
                          attributes: Vec<IssueAttribute>) -> APIResult<HashMap<String, IssueData>> {
        let mut map = Map::new();
        map.insert(
            "issue_ids".to_string(),
            Value::Array(issues.into_iter().map(Value::String).collect())
        );
        map.insert(
            "attributes".to_string(),
            Value::Array(attributes.into_iter().map(|s| Value::String(s.to_string())).collect())
        );
        #[derive(Debug, serde::Deserialize)]
        struct IssueDataResponse {
            data: HashMap<String, RawIssueData>
        }
        let response = self.call_endpoint_json::<_, IssueDataResponse>(
            "issue-data", Verb::Get, Value::Object(map)
        )?.data;
        let result = response
            .into_iter()
            .map(|(ident, data)| (
                ident.clone(),
                IssueData::from_raw_data(ident, data)
            ))
            .collect();
        Ok(result)
    }

    /***************************************************************************
     * Tag-related endpoints
     */

    pub(crate) fn get_all_tags(&self) -> APIResult<Vec<UnboundTag>> {
        #[derive(Debug, serde::Deserialize)]
        struct TagListResponse {
            tags: Vec<UnboundTag>
        }
        let result = self.call_endpoint_json::<_, TagListResponse>(
            "tags", Verb::Get, Value::Object(Map::new())
        )?;
        Ok(result.tags)
    }

    pub(crate) fn register_new_tag(&self, name: String, description: String) -> APIResult<()> {
        let mut map = Map::new();
        map.insert("tag".to_string(), Value::String(name));
        map.insert("description".to_string(), Value::String(description));
        self.call_endpoint_json("tags", Verb::Post, Value::Object(map))?;
        Ok(())
    }

    pub(crate) fn get_tag_info(&self, tag: String) -> APIResult<UnboundTag> {
        #[derive(Debug, serde::Deserialize)]
        struct TagInfoResponse {
            tag: UnboundTag
        }
        let endpoint = format!("tags/{}", tag);
        let result = self.call_endpoint_json::<_, TagInfoResponse>(
            endpoint.as_str(), Verb::Get, Value::Object(Map::new())
        )?;
        Ok(result.tag)
    }

    pub(crate) fn update_tag(&self, tag: String, new_description: String) -> APIResult<()> {
        let mut map = Map::new();
        map.insert("description".to_string(), Value::String(new_description));
        let endpoint = format!("tags/{}", tag);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Post, Value::Object(map)
        )?;
        Ok(())
    }

    pub(crate) fn delete_tag(&self, tag: String) -> APIResult<()> {
        let endpoint = format!("tags/{}", tag);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Delete, Value::Object(Map::new())
        )?;
        Ok(())
    }

    /***************************************************************************
     * Tag + Issue endpoints
     */

    pub(crate) fn start_issue_review(&self, issue_id: String) -> APIResult<()> {
        let endpoint = format!("issues/{}/mark-review", issue_id);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Post, Value::Object(Map::new())
        )?;
        Ok(())
    }

    pub(crate) fn finish_issue_review(&self, issue_id: String) -> APIResult<()> {
        let endpoint = format!("issues/{}/finish-review", issue_id);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Post, Value::Object(Map::new())
        )?;
        Ok(())
    }

    pub(crate) fn get_tags_for_issue(&self, issue_id: String) -> APIResult<Vec<String>> {
        #[derive(Debug, serde::Deserialize)]
        struct TagsResponse {
            tags: Vec<String>
        }
        let endpoint = format!("issues/{}/tags", issue_id);
        let result = self.call_endpoint_json::<_, TagsResponse>(
            endpoint.as_str(), Verb::Get, Value::Object(Map::new())
        )?;
        Ok(result.tags)
    }

    pub(crate) fn add_tag_to_issue(&self, issue_id: String, tag: String) -> APIResult<()> {
        let endpoint = format!("issues/{}/tags", issue_id);
        let mut map = Map::new();
        map.insert("tag".to_string(), Value::String(tag));
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Post, map
        )?;
        Ok(())
    }

    pub(crate) fn remove_tag_from_issue(&self, issue_id: String, tag: String) -> APIResult<()> {
        let endpoint = format!("issues/{}/tags/{}", issue_id, tag);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Delete, Value::Object(Map::new())
        )?;
        Ok(())
    }

    pub(crate) fn bulk_add_tags(&self, issues_and_tags: HashMap<String, Vec<String>>) -> APIResult<()> {
        let maps = issues_and_tags
            .into_iter()
            .map(|(key, value)| {
                let mut map = Map::new();
                map.insert("issue_id".to_string(), Value::String(key));
                let tags = value.into_iter().map(Value::String).collect();
                map.insert("tags".to_string(), Value::Array(tags));
                Value::Object(map)
            }).collect::<Vec<_>>();
        let mut map = Map::new();
        map.insert("data".to_string(), Value::Array(maps));
        self.call_endpoint_json(
            "/bulk/add-tags", Verb::Post, Value::Object(map)
        )
    }

    /***************************************************************************
     * Labeling endpoints
     */

    pub(crate) fn get_labeling_comments_for_issue(&self, issue_id: String) -> APIResult<Vec<UnboundComment>> {
        #[derive(Debug, serde::Deserialize)]
        struct RawComment {
            author: String,
            comment: String
        }
        #[derive(Debug, serde::Deserialize)]
        struct CommentsResponse {
            comments: HashMap<String, RawComment>
        }
        let endpoint = format!("manual-labels/{}/comments", issue_id);
        let result = self.call_endpoint_json::<_, CommentsResponse>(
            endpoint.as_str(), Verb::Get, Value::Object(Map::new())
        )?;
        // collect converted comments
        let converted: Vec<UnboundComment> = result.comments.into_iter()
            .map(|(id, raw)| UnboundComment{id, author: raw.author, text: raw.comment})
            .collect();
        // Get object IDS in order to restore comment order
        let mut object_ids: Vec<u128> = Vec::with_capacity(converted.len());
        for c in converted.iter() {
            // let id = c.id
            //     .parse::<u128>()
            //     .map_err(|e| IDParsingError{msg: e.to_string()})?;
            let id = u128::from_str_radix(c.id.as_str(), 16)
                .map_err(|e| APIError::IDParsingError(e.to_string()))?;
            object_ids.push(id);
        }
        // Sort comments
        let mut pairs = object_ids.into_iter()
            .zip(converted.into_iter()).collect::<Vec<_>>();
        pairs.sort_by_key(|p| p.0);
        // Extract comments without ids
        let (_, comments): (Vec<u128>, Vec<UnboundComment>) = pairs.into_iter().unzip();
        Ok(comments)
    }

    pub(crate) fn add_labeling_comment_to_issue(&self, issue_id: String, text: String) -> APIResult<String> {
        #[derive(Debug, serde::Deserialize)]
        struct NewCommentResponse {
            comment_id: String
        }
        let endpoint = format!("manual-labels/{}/comments", issue_id);
        let mut map = Map::new();
        map.insert("comment".to_string(), Value::String(text));
        let result = self.call_endpoint_json::<_, NewCommentResponse>(
            endpoint.as_str(), Verb::Post, Value::Object(map)
        )?;
        Ok(result.comment_id)
    }

    pub(crate) fn update_labeling_comment(&self,
                                   issue_id: String,
                                   comment_id: String,
                                   new_text: String) -> APIResult<()> {
        let endpoint = format!("manual-labels/{}/comments/{}", issue_id, comment_id);
        let mut map = Map::new();
        map.insert("comment".to_string(), Value::String(new_text));
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Patch, Value::Object(map)
        )?;
        Ok(())
    }

    pub(crate) fn delete_labeling_comment(&self,
                                   issue_id: String,
                                   comment_id: String) -> APIResult<()> {
        let endpoint = format!("manual-labels/{}/comments/{}", issue_id, comment_id);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Delete, Value::Object(Map::new())
        )?;
        Ok(())
    }

    pub(crate) fn get_manual_labels(&self, issues: Vec<String>) -> APIResult<HashMap<String, Label>> {
        #[derive(Debug, serde::Deserialize)]
        struct LabelsResponse {
            manual_labels: HashMap<String, Label>
        }
        let mut map = Map::new();
        map.insert("issue_ids".to_string(),
                   Value::Array(issues.into_iter().map(Value::String).collect()));
        let result = self.call_endpoint_json::<_, LabelsResponse>(
            "manual-labels", Verb::Get, map
        )?;
        Ok(result.manual_labels)
    }

    pub(crate) fn update_manual_label_for_issue(&self, issue_id: String, label: Label) -> APIResult<()> {
        let endpoint = format!("manual-labels/{}", issue_id);
        self.call_endpoint_json(endpoint.as_str(), Verb::Post, label)?;
        Ok(())
    }

    /***************************************************************************
     * Embedding-related endpoints
     */

    pub(crate) fn get_all_embeddings(&self) -> APIResult<Vec<UnboundEmbedding>> {
        #[derive(Debug, serde::Deserialize)]
        struct EmbeddingsResponse {
            embeddings: Vec<UnboundEmbedding>
        }
        let result = self.call_endpoint_json::<_, EmbeddingsResponse>(
            "embeddings", Verb::Get, Value::Object(Map::new())
        )?;
        Ok(result.embeddings)
    }

    pub(crate) fn create_embedding(&self,
                            name: String,
                            config: HashMap<String, Value>) -> APIResult<String> {
        let mut map = Map::new();
        map.insert("name".to_string(), Value::String(name));
        map.insert("config".to_string(),
                   Value::Object(Map::from_iter(config.into_iter())));
        let result = self.call_endpoint_json::<_, String>(
            "embeddings", Verb::Post, Value::Object(map)
        )?;
        Ok(result)
    }

    pub(crate) fn get_embedding(&self, id: String) -> APIResult<UnboundEmbedding> {
        let endpoint = format!("embeddings/{}", id);
        self.call_endpoint_json::<_, UnboundEmbedding>(
            endpoint.as_str(), Verb::Get, Value::Object(Map::new())
        )
    }

    pub(crate) fn update_embedding(&self,
                            id: String,
                            name: String,
                            config: HashMap<String, Value>) -> APIResult<()> {
        let mut map = Map::new();
        map.insert("name".to_string(), Value::String(name));
        map.insert("config".to_string(),
                   Value::Object(Map::from_iter(config.into_iter())));
        let endpoint = format!("embeddings/{}", id);
        self.call_endpoint_json(endpoint.as_str(), Verb::Post, map)?;
        Ok(())
    }


    pub(crate) fn delete_embedding(&self, id: String) -> APIResult<()> {
        let endpoint = format!("embeddings/{}", id);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Delete, Value::Object(Map::new())
        )?;
        Ok(())
    }

    pub(crate) fn upload_embedding_binary(&self, id: String, filename: String) -> APIResult<()> {
        let form = multipart::Form::new().file("file", filename)?;
        let endpoint = format!("embeddings/{}/file", id);
        self.call_endpoint_multipart(endpoint.as_str(), Verb::Post, form)
    }

    pub(crate) fn download_embedding_binary(&self, id: String, filename: String) -> APIResult<()> {
        let endpoint = format!("embeddings/{}/file", id);
        self.call_endpoint_download(endpoint.as_str(),
                                    Verb::Get,
                                    Value::Object(Map::new()),
                                    filename)
    }

    pub(crate) fn delete_embedding_binary(&self, id: String) -> APIResult<()> {
        let endpoint = format!("embeddings/{}/file", id);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Delete, Value::Object(Map::new())
        )?;
        Ok(())
    }

    /***************************************************************************
     * Repos and Project endpoints
     */

    pub(crate) fn get_all_repos(&self) -> APIResult<Vec<String>> {
        #[derive(Debug, serde::Deserialize)]
        struct ReposResponse {
            repos: Vec<String>
        }
        let result = self.call_endpoint_json::<_, ReposResponse>(
            "repos", Verb::Get, Value::Object(Map::new())
        )?;
        Ok(result.repos)
    }

    pub(crate) fn get_projects_for_repo(&self, repo: String) -> APIResult<Vec<String>> {
        #[derive(Debug, serde::Deserialize)]
        struct ProjectsResponse {
            projects: Vec<String>
        }
        let endpoint = format!("repos/{}/projects", repo);
        let result = self.call_endpoint_json::<_, ProjectsResponse>(
            endpoint.as_str(), Verb::Get, Value::Object(Map::new())
        )?;
        Ok(result.projects)
    }

    /***************************************************************************
     * Model Config Endpoints
     */

    pub(crate) fn get_all_models(&self) -> APIResult<Vec<ModelInfo>> {
        #[derive(Debug, serde::Deserialize)]
        struct ModelsResponse {
            models: Vec<ModelInfo>
        }
        let result = self.call_endpoint_json::<_, ModelsResponse>(
            "models", Verb::Get, Value::Object(Map::new())
        )?;
        Ok(result.models)
    }

    pub(crate) fn create_model_config(&self,
                                      name: String,
                                      config: HashMap<String, Value>) -> APIResult<String> {
        #[derive(Debug, serde::Deserialize)]
        struct NewModelResponse {
            model_id: String
        }
        let mut map = Map::new();
        map.insert("model_name".to_string(), Value::String(name));
        map.insert(
            "model_config".to_string(),
            Value::Object(Map::from_iter(config.into_iter()))
        );
        let result = self.call_endpoint_json::<_, NewModelResponse>(
            "models", Verb::Post, Value::Object(map)
        )?;
        Ok(result.model_id)
    }

    pub(crate) fn get_model_config(&self, id: String) -> APIResult<UnboundModelConfig> {
        let endpoint = format!("models/{}", id);
        self.call_endpoint_json::<_, UnboundModelConfig>(
            endpoint.as_str(), Verb::Get, Value::Object(Map::new())
        )
    }

    pub(crate) fn update_model_config(&self,
                                      id: String,
                                      name: String,
                                      config: HashMap<String, Value>) -> APIResult<()> {
        let endpoint = format!("models/{}", id);
        let mut map = Map::new();
        map.insert("model_name".to_string(), Value::String(name));
        map.insert("model_config".to_string(),
                   Value::Object(Map::from_iter(config.into_iter())));
        self.call_endpoint_json(endpoint.as_str(), Verb::Post, Value::Object(map))
    }

    pub(crate) fn delete_model_config(&self, id: String) -> APIResult<()> {
        let endpoint = format!("models/{}", id);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Delete, Value::Object(Map::new())
        )
    }

    /***************************************************************************
     * Model Versions
     */

    pub(crate) fn get_versions_for_model(&self,
                                         model_id: String) -> APIResult<Vec<UnboundModelVersion>> {
        let endpoint = format!("models/{}/versions", model_id.as_str());
        #[derive(Debug, serde::Deserialize)]
        struct RawVersionInfo {
            version_id: String,
            description: String
        }
        #[derive(Debug, serde::Deserialize)]
        struct VersionsResponse {
            versions: Vec<RawVersionInfo>
        }
        let result = self.call_endpoint_json::<_, VersionsResponse>(
            endpoint.as_str(), Verb::Get, Value::Object(Map::new())
        )?;
        let converted = result.versions
            .into_iter()
            .map(|v|
                UnboundModelVersion{
                    model_id: model_id.clone(),
                    version_id: v.version_id,
                    description: v.description
                }
            )
            .collect();
        Ok(converted)
    }

    pub(crate) fn upload_model_version(&self,
                                       model_id: String,
                                       file: String) -> APIResult<String> {
        #[derive(Debug, serde::Deserialize)]
        struct NewVersionResponse {
            version_id: String
        }
        let form = multipart::Form::new()
            .file("file", file)?;
        let endpoint = format!("models/{}/versions", model_id);
        let result = self.call_endpoint_multipart::<NewVersionResponse>(
            endpoint.as_str(), Verb::Post, form
        )?;
        Ok(result.version_id)
    }

    pub(crate) fn download_model_version(&self,
                                         model_id: String,
                                         version_id: String,
                                         filename: String) -> APIResult<()> {
        let endpoint = format!("models/{}/versions/{}", model_id, version_id);
        self.call_endpoint_download(endpoint.as_str(),
                                    Verb::Get,
                                    Value::Object(Map::new()),
                                    filename)
    }

    pub(crate) fn delete_model_version(&self,
                                       model_id: String,
                                       version_id: String) -> APIResult<()> {
        let endpoint = format!("models/{}/versions/{}", model_id, version_id);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Delete, Value::Object(Map::new())
        )
    }

    pub(crate) fn update_version_description(&self,
                                             model_id: String,
                                             version_id: String,
                                             description: String) -> APIResult<()> {
        let endpoint = format!("models/{}/versions/{}/description", model_id, version_id);
        let mut map = Map::new();
        map.insert("description".to_string(), Value::String(description));
        self.call_endpoint_json(endpoint.as_str(), Verb::Put, Value::Object(map))
    }

    /***************************************************************************
    * Model Predictions
    */

    pub(crate) fn get_predictions(&self,
                                  model_id: String,
                                  version_id: String,
                                  issues: Option<Vec<String>>) -> APIResult<HashMap<String, Value>> {
        let endpoint = format!("models/{}/versions/{}/predictions", model_id, version_id);
        let mut map = Map::new();
        let payload = match issues {
            None => Value::Null,
            Some(ids) => Value::Array(ids.into_iter().map(Value::String).collect())
        };
        map.insert("issue_ids".to_string(), payload);
        #[derive(Debug, serde::Deserialize)]
        struct PredictionsResponse {
            predictions: HashMap<String, Value>
        }
        //let result = self.call_endpoint_json::<_, PredictionsResponse>(
        //    endpoint.as_str(), Verb::Get, Value::Object(map)
        //)?;
        let result = self.call_endpoint_download_object::<_, PredictionsResponse>(
            endpoint.as_str(), Verb::Get, Value::Object(map)
        )?;
        Ok(result.predictions)
    }

    pub(crate) fn store_predictions(&self,
                                    model_id: String,
                                    version_id: String,
                                    predictions: HashMap<String, Value>) -> APIResult<()> {
        let endpoint = format!("models/{}/versions/{}/predictions", model_id, version_id);
        let mut map = Map::new();
        map.insert("predictions".to_string(),
                   Value::Object(Map::from_iter(predictions.into_iter())));
        //self.call_endpoint_json(endpoint.as_str(), Verb::Post, Value::Object(map))
        self.call_endpoint_multipart_object(
            endpoint.as_str(), Verb::Post, &Value::Object(map)
        )
    }

    pub(crate) fn delete_predictions(&self,
                                     model_id: String,
                                     version_id: String) -> APIResult<()> {
        let endpoint = format!("models/{}/versions/{}/predictions", model_id, version_id);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Delete, Value::Object(Map::new())
        )
    }

    /***************************************************************************
     * Model Performance
     */

    pub(crate) fn get_performances_for_model(&self, model_id: String) -> APIResult<Vec<UnboundTestRun>> {
        #[derive(Debug, serde::Deserialize)]
        struct PerformancesInfo {
            performance_id: String,
            description: String
        }
        #[derive(Debug, serde::Deserialize)]
        struct PerformancesResponse {
            performances: Vec<PerformancesInfo>
        }
        let endpoint = format!("models/{}/performances", model_id.as_str());
        let result = self.call_endpoint_json::<_, PerformancesResponse>(
            endpoint.as_str(), Verb::Get, Value::Object(Map::new())
        )?;
        let converted = result.performances
            .into_iter()
            .map(|r| UnboundTestRun{
                model_id: model_id.clone(),
                performance_id: r.performance_id,
                description: r.description
            }).collect();
        Ok(converted)
    }

    pub(crate) fn store_model_performance(&self,
                                          model_id: String,
                                          data: Vec<Value>) -> APIResult<String> {
        #[derive(Debug, serde::Deserialize)]
        struct NewPerformanceResponse {
            performance_id: String
        }
        //let mut map = Map::new();
        //map.insert("performance".to_string(), Value::Array(data));
        let endpoint = format!("models/{}/performances", model_id);
        // let result = self.call_endpoint_json::<_, NewPerformanceResponse>(
        //     endpoint.as_str(), Verb::Post, Value::Object(map)
        // )?;
        let result = self.call_endpoint_multipart_object::<_, NewPerformanceResponse>(
            endpoint.as_str(), Verb::Post, &data
        )?;
        Ok(result.performance_id)
    }

    pub(crate) fn get_performance_data(&self,
                                       model_id: String,
                                       performance_id: String) -> APIResult<Vec<Value>> {
        #[derive(Debug, serde::Deserialize)]
        struct PerformanceDataResponse {
            performance_id: String,
            description: String,
            performance: Vec<Value>
        }
        let endpoint = format!("models/{}/performances/{}", model_id, performance_id);
        // let result = self.call_endpoint_json::<_, PerformanceDataResponse>(
        //     endpoint.as_str(), Verb::Get, Value::Object(Map::new())
        // )?;
        let result = self.call_endpoint_download_object::<_, PerformanceDataResponse>(
            endpoint.as_str(), Verb::Get, Value::Object(Map::new())
        )?;
        Ok(result.performance)
    }

    pub(crate) fn delete_performance_data(&self,
                                          model_id: String,
                                          performance_id: String) -> APIResult<()> {
        let endpoint = format!("models/{}/performances/{}", model_id, performance_id);
        self.call_endpoint_json(
            endpoint.as_str(), Verb::Delete, Value::Object(Map::new())
        )
    }

    pub(crate) fn update_performance_description(&self,
                                                 model_id: String,
                                                 performance_id: String,
                                                 description: String) -> APIResult<()> {
        let endpoint = format!("models/{}/performances/{}/description", model_id, performance_id);
        let mut map = Map::new();
        map.insert("description".to_string(), Value::String(description));
        self.call_endpoint_json(endpoint.as_str(), Verb::Put, Value::Object(map))
    }
}
