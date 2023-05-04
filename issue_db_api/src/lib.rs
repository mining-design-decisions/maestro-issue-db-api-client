mod repository;
mod api_core;
mod schemas;
mod query;
mod labels;
mod util;
mod tags;
mod embedding;
mod comments;
mod issues;
mod config;
mod errors;
mod models;

pub use repository::IssueRepository;
pub use errors::APIResult;
pub use query::{Query, QueryCMP};


#[cfg(feature = "pyo3")]
mod python {
    use std::collections::HashMap;
    use std::hash::{Hash, Hasher};
    use pyo3::basic::CompareOp;
    use pyo3::prelude::*;
    use pyo3::create_exception;
    use pyo3::exceptions::{PyException, PyTypeError, PyValueError};
    use pyo3::types::{IntoPyDict, PyBool, PyDict, PyFloat, PyInt, PyList, PyLong, PyString, PyTuple};
    use serde_json::{Map, Number, Value};
    use crate::comments::Comment;
    use crate::config::{CachingPolicy, ConfigHandlingPolicy, IssueAttribute, IssueLoadingSettings};
    use crate::embedding::Embedding;
    use crate::errors::APIError;
    use crate::issues::Issue;
    use crate::labels::Label;
    use crate::models::{Model, ModelVersion, TestRun};
    use crate::repository::IssueRepo;
    use crate::tags::{Tag, TagType};
    use crate::errors::APIResult;
    use super::*;

    create_exception!(issue_api, IssueAPIError, PyException);
    create_exception!(issue_api, InvalidCredentialsException, IssueAPIError);
    create_exception!(issue_api, NotAuthorizedException, IssueAPIError);
    create_exception!(issue_api, InvalidTokenException, IssueAPIError);
    create_exception!(issue_api, HTTPException, IssueAPIError);
    create_exception!(issue_api, LibraryException, IssueAPIError);

    #[inline(always)]
    fn api2py_error<T>(e: APIResult<T>) -> PyResult<T> {
        match e {
            Ok(obj) => Ok(obj),
            Err(inner) => Err({
                match inner {
                    APIError::InvalidCredentials => InvalidCredentialsException::new_err(inner.to_string()),
                    APIError::InvalidToken => InvalidTokenException::new_err(inner.to_string()),
                    APIError::NotAuthorized => NotAuthorizedException::new_err(inner.to_string()),
                    APIError::HTTPError{message, status_code} => {
                        let o = (message, status_code);
                        HTTPException::new_err(o)
                    },
                    APIError::IDParsingError(_) => IssueAPIError::new_err(inner.to_string()),
                    APIError::LibraryError(_) => LibraryException::new_err(inner.to_string()),
                    APIError::GenericError(_) => IssueAPIError::new_err(inner.to_string())
                }
            })
        }
    }

    fn json_to_py(py: Python<'_>, j: Value) -> PyObject {
        match j {
            Value::Null => Option::<bool>::None.into_py(py),
            Value::Bool(b) => b.into_py(py),
            Value::Number(n) => {
                if n.is_f64() {
                    n.as_f64().unwrap().into_py(py)
                } else if n.is_i64() {
                    n.as_i64().into_py(py)
                } else {    // is_u64
                    n.as_u64().into_py(py)
                }
            }
            Value::String(s) => s.into_py(py),
            Value::Array(v) => {
                v.into_iter()
                    .map(|x| json_to_py(py, x))
                    .collect::<Vec<_>>()
                    .into_py(py)
            }
            Value::Object(o) => {
                 o.into_iter()
                     .map(|(k, v)| (k, json_to_py(py, v)))
                     .collect::<HashMap<_, _>>()
                     .into_py_dict(py)
                     .into()
            }
        }
    }

    fn py_to_json(obj: &PyAny) -> PyResult<Value> {
        #[allow(clippy::if_same_then_else)]
        if obj.is_instance_of::<PyDict>()? {
            let mut map = Map::new();
            for (key, value) in obj.downcast::<PyDict>()? {
                let k = key.extract::<String>()?;
                let v = py_to_json(value)?;
                map.insert(k, v);
            }
            Ok(Value::Object(map))
        } else if obj.is_instance_of::<PyList>()? {
            let mut items = Vec::new();
            for o in obj.downcast::<PyList>()? {
                items.push(py_to_json(o)?);
            }
            Ok(Value::Array(items))
        } else if obj.is_instance_of::<PyFloat>()? {
            let num = Number::from_f64(obj.extract::<f64>()?);
            match num {
                None => {
                    let msg = format!("Failed to convert float: {}", obj);
                    Err(PyValueError::new_err(msg))
                }
                Some(f) => Ok(Value::Number(f))
            }
        } else if obj.is_instance_of::<PyBool>()? {     // Must be before int because issubclass(bool, int)
            Ok(Value::Bool(obj.extract::<bool>()?))
        } else if obj.is_instance_of::<PyInt>()? {
            Ok(Value::Number(Number::from(obj.extract::<i64>()?)))
        } else if obj.is_instance_of::<PyLong>()? {
            Ok(Value::Number(Number::from(obj.extract::<i64>()?)))
        } else if obj.is_instance_of::<PyString>()? {
            Ok(Value::String(obj.extract::<String>()?))
        } else if obj.is_none() {
           Ok(Value::Null)
        } else {
            let msg = format!("Cannot convert \"{}\" object to JSON", obj.get_type().name()?);
            Err(PyTypeError::new_err(msg))
        }
    }

    fn parse_issue_attributes(attributes: Vec<String>) -> PyResult<Vec<IssueAttribute>> {
        let mut converted: Vec<IssueAttribute> = Vec::with_capacity(attributes.len());
        for attr in attributes {
            converted.push(api2py_error(attr.try_into())?);
        }
        Ok(converted)
    }

    #[pyclass(name="IssueRepository")]
    #[allow(unused)]
    struct PyIssueRepository {
        url: String,
        authenticated: bool,
        repo: IssueRepository
    }

    #[pymethods]
    impl PyIssueRepository {
        #[new]
        #[pyo3(signature=(
            url, *,
            credentials=None,
            label_caching_policy="no_caching",
            config_handling_policy="read_fetch_write_fetch",
            allow_self_signed_certificates=false
        ))]
        fn __new__(url: String,
                   credentials: Option<(String, String)>,
                   label_caching_policy: &str,
                   config_handling_policy: &str,
                   allow_self_signed_certificates: bool) -> PyResult<Self> {
            let caching = match label_caching_policy {
                "no_caching" => CachingPolicy::NoCaching,
                "use_local_after_load" => CachingPolicy::UseLocalAfterLoad,
                _ => {
                    let text = format!("Invalid caching policy: {}", label_caching_policy);
                    return Err(IssueAPIError::new_err(text));
                }
            };
            let config_handling = match config_handling_policy {
                "read_fetch_write_fetch" => ConfigHandlingPolicy::ReadFetchWriteWithFetch,
                "read_local_write_fetch" => ConfigHandlingPolicy::ReadLocalWriteWithFetch,
                "read_local_write_local" => ConfigHandlingPolicy::ReadLocalWriteNoFetch,
                _ => {
                    let text = format!("Invalid config handling policy: {}", config_handling_policy);
                    return Err(IssueAPIError::new_err(text));
                }
            };
            let (repo, auth) = if let Some((username, password)) = credentials {
                (
                    IssueRepository::new(url.clone(),
                                         username,
                                         password,
                                         caching,
                                         config_handling,
                                         allow_self_signed_certificates),
                    true
                )
            } else {
                (
                    IssueRepository::new_read_only(url.clone(),
                                                   caching,
                                                   config_handling,
                                                   allow_self_signed_certificates),
                    false
                )
            };
            Ok(Self{url, authenticated: auth, repo: api2py_error(repo)?})
        }

        fn __repr__(&self) -> PyResult<String> {
            let text = format!(
                "<IssueRepository|url={}, authenticated={}>", self.url, self.authenticated
            );
            Ok(text)
        }

        #[pyo3(signature=(/, q, *, attributes=vec![], load_labels=false))]
        fn search(&self, q: PyQuery, attributes: Vec<String>, load_labels: bool) -> PyResult<Vec<PyIssue>> {
            let settings = IssueLoadingSettings::new(
                parse_issue_attributes(attributes)?, load_labels
            );
            let query = q.get_query()?;
            let result = self.repo.search(query, settings);
            let issues = api2py_error(result)?;
            let py_issues = issues.into_iter()
                .map(|issue| PyIssue{issue})
                .collect();
            Ok(py_issues)
        }

        #[getter]
        fn repos(&self) -> PyResult<Vec<PyRepo>> {
            let repos = api2py_error(self.repo.repos())?
                .into_iter()
                .map(|r| PyRepo{inner: r})
                .collect();
            Ok(repos)
        }

        #[getter]
        fn tags(&self) -> PyResult<Vec<PyTag>> {
            let tags = api2py_error(self.repo.tags())?
                .into_iter()
                .map(|t| PyTag{inner: t})
                .collect();
            Ok(tags)
        }

        fn add_new_tag(&self, name: String, description: String) -> PyResult<()> {
            api2py_error(self.repo.add_new_tag(name, description))
        }

        // fn bulk_add_tags(&self, i: &Vec<PyIssue>) -> PyResult<()> {
        //     let payload = tags_per_issue.into_iter()
        //         .map(|(k, v)| (&k.issue, v))
        //         .collect();
        //     api2py_error(self.repo.bulk_add_tags(payload))
        // }

        #[getter]
        fn embeddings(&self) -> PyResult<Vec<PyEmbedding>> {
            let embeddings = api2py_error(self.repo.embeddings())?
                .into_iter()
                .map(|e| PyEmbedding{inner: e})
                .collect();
            Ok(embeddings)
        }

        fn get_embedding_by_id(&self, id: String) -> PyResult<PyEmbedding> {
            let e = PyEmbedding{inner: api2py_error(self.repo.get_embedding_by_id(id))?};
            Ok(e)
        }

        fn create_embedding(&self, name: String, config: &PyAny) -> PyResult<PyEmbedding> {
            let json = py_to_json(config)?;
            if let Value::Object(c) = json {
                let config = c.into_iter().collect();
                Ok(PyEmbedding{inner: api2py_error(self.repo.create_embedding(name, config))?})
            } else {
                Err(PyTypeError::new_err("Top-level JSON must be an object"))
            }
        }

        fn delete_embedding(&self, embedding: &PyEmbedding) -> PyResult<()> {
            api2py_error(self.repo.delete_embedding(embedding.inner.clone()))
        }

        #[pyo3(signature=(*args, attributes=vec![], load_labels=false))]
        fn find_issues_by_key(&self,
                              args: &PyTuple,
                              attributes: Vec<String>,
                              load_labels: bool) -> PyResult<Vec<PyIssue>> {
            let keys = args.extract::<Vec<(String, String)>>()?;
            let settings = IssueLoadingSettings::new(
                parse_issue_attributes(attributes)?, load_labels
            );
            let issues = api2py_error(self.repo.find_issues_by_key(keys, settings))?;
            Ok(issues.into_iter().map(|i| PyIssue{issue: i}).collect())
        }

        #[getter]
        fn models(&self) -> PyResult<Vec<PyModel>> {
            let result = api2py_error(self.repo.models())?
                .into_iter()
                .map(|m| PyModel{inner: m})
                .collect();
            Ok(result)
        }

        fn get_model_by_id(&self, id: String) -> PyResult<PyModel> {
            let m = PyModel{inner: api2py_error(self.repo.get_model_by_id(id))?};
            Ok(m)
        }

        fn add_model(&self, name: String, config: HashMap<String, &PyAny>) -> PyResult<PyModel> {
            let mut converted = HashMap::with_capacity(config.len());
            for (k, v) in config {
                converted.insert(k, py_to_json(v)?);
            }
            let model = api2py_error(self.repo.add_model(name, converted))?;
            Ok(PyModel{inner: model})
        }

        fn delete_model_config(&self, model: &PyModel) -> PyResult<()> {
            api2py_error(self.repo.delete_model_config_by_id(model.inner.identifier()))

        }
    }

    #[pyclass(name="IssueRepo")]
    struct PyRepo {
        inner: IssueRepo
    }

    #[pymethods]
    impl PyRepo {
        fn __repr__(&self) -> PyResult<String> {
            Ok(format!("<IssueRepo|name={}>", self.inner.name()))
        }

        #[getter]
        fn name(&self) -> PyResult<String> {
            Ok(self.inner.name().clone())
        }

        #[getter]
        fn projects(&self) -> PyResult<Vec<String>> {
            api2py_error(self.inner.projects())
        }
    }

    #[pyclass(name="Query")]
    #[allow(unused)]
    #[derive(Clone)]
    pub struct PyQuery {
        query: Option<Query>
    }

    impl PyQuery {
        fn collect_branches(&self, args: &PyTuple) -> PyResult<Vec<Query>> {
            let mut branches: Vec<Query> = Vec::new();
            for o in args.iter() {
                let q = o.extract::<PyQuery>()?;    // implicitly clones
                if let Some(raw) = q.query {
                    branches.push(raw);
                }
            }
            if let Some(ref raw) = self.query {
                branches.push(raw.clone());
            }
            Ok(branches)
        }

        fn add_tag(&self, name: String, cmp: QueryCMP) -> PyResult<Query> {
            if self.query.is_some() {
                return Err(
                    IssueAPIError::new_err(
                        "issues_api.Query.tag cannot be called on a tag-query."
                    )
                )
            }
            Ok(Query::Tag(cmp, name))
        }
        
        fn add_exists(&self, name: String, state: bool) -> PyResult<Query> {
            if self.query.is_some() {
                return Err(
                    IssueAPIError::new_err(
                        "issues_api.Query.tag cannot be called on a tag-query."
                    )
                )
            }
            Ok(Query::Exists(name, state))
        }

        fn get_query(&self) -> PyResult<Query> {
            match self.query {
                None => Err(
                    IssueAPIError::new_err("Cannot perform search with empty query")
                ),
                Some(ref q) => Ok(q.clone())
            }
        }
    }

    #[pymethods]
    impl PyQuery {
        #[new]
        fn __new__() -> PyResult<Self> {
            Ok(Self{query: None})
        }

        fn __repr__(&self) -> PyResult<String> {
            match self.query {
                None => Ok("<Query|query=<empty>>".to_string()),
                Some(ref q) => Ok(format!("<Query|query={}>", q))
            }
        }

        #[pyo3(signature=(*args))]
        fn lor(&self, args: &PyTuple) -> PyResult<Self> {
            let branches = self.collect_branches(args)?;
            Ok(Self{query: Some(Query::Or(branches))})
        }

        #[pyo3(signature=(*args))]
        fn land(&self, args: &PyTuple) -> PyResult<Self> {
            let branches = self.collect_branches(args)?;
            Ok(Self{query: Some(Query::And(branches))})
        }

        fn tag(&self, name: String) -> PyResult<Self> {
            Ok(Self{query: Some(self.add_tag(name, QueryCMP::Eq)?)})
        }

        fn not_tag(&self, name: String) -> PyResult<Self> {
            Ok(Self{query: Some(self.add_tag(name, QueryCMP::Ne)?)})
        }
        
        fn exists(&self, field: String) -> PyResult<Self> {
            Ok(
                Self{query: Some(self.add_exists(field, true)?)}
            )
        }
        
        fn not_exists(&self, field: String) -> PyResult<Self> {
            Ok(
                Self{query: Some(self.add_exists(field, false)?)}
            )
        }

        fn to_json(&self, py: Python<'_>) -> PyResult<PyObject> {
            match self.query {
                None => Ok(py.None()),
                Some(ref q) => Ok(json_to_py(py, q.clone().into_json()))
            }
        }
    }

    #[pyclass(name="Label")]
    #[allow(unused)]
    #[derive(Debug, Copy, Clone)]
    #[derive(Eq, PartialEq, Hash)]
    struct PyLabel {
        inner: Label
    }

    #[pymethods]
    impl PyLabel {
        #[new]
        fn __new__(existence: bool, executive: bool, property: bool) -> PyResult<Self> {
            Ok(Self{inner: Label::new(existence, executive, property)})
        }

        fn __repr__(&self) -> PyResult<String> {
            let text = format!(
                "issue_api.Label(existence={}, executive={}, property={})",
                if self.inner.existence() { "True" } else { "False" },
                if self.inner.executive() { "True" } else { "False" },
                if self.inner.property() { "True" } else { "False" }
            );
            Ok(text)
        }

        fn __richcmp__(&self, other: PyRef<PyLabel>, op: CompareOp) -> Py<PyAny> {
            let py = other.py();
            match op {
                CompareOp::Eq => (self.inner == other.inner).into_py(py),
                CompareOp::Ne => (self.inner != other.inner).into_py(py),
                _ => py.NotImplemented(),
            }
        }

        fn __hash__(&self) -> PyResult<u64> {
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.inner.hash(&mut s);
            Ok(s.finish())
        }

        #[getter]
        fn existence(&self) -> PyResult<bool> {
            Ok(self.inner.existence())
        }

        #[getter]
        fn executive(&self) -> PyResult<bool> {
            Ok(self.inner.executive())
        }

        #[getter]
        fn property(&self) -> PyResult<bool> {
            Ok(self.inner.property())
        }

        #[getter]
        fn non_architectural(&self) -> PyResult<bool> {
            Ok(
                !(self.inner.existence() || self.inner.executive() || self.inner.property())
            )
        }
    }

    #[pyclass(name="Issue")]
    #[allow(unused)]
    #[derive(Debug, PartialEq, Eq, Hash)]
    struct PyIssue {
        issue: Issue
    }

    #[pymethods]
    impl PyIssue {
        #[new]
        fn __new__() -> PyResult<Self> {
            Err(IssueAPIError::new_err("issue_api.Issue cannot be instantiated directly"))
        }

        fn __repr__(&self) -> PyResult<String> {
            Ok(format!("<Issue|id={}>", self.issue.ident()))
        }

        fn __richcmp__(&self, other: PyRef<Self>, op: CompareOp) -> Py<PyAny> {
            let py = other.py();
            match op {
                CompareOp::Eq => (self.issue == other.issue).into_py(py),
                CompareOp::Ne => (self.issue != other.issue).into_py(py),
                _ => py.NotImplemented(),
            }
        }

        fn __hash__(&self) -> PyResult<u64> {
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.issue.hash(&mut s);
            Ok(s.finish())
        }

        #[getter]
        fn identifier(&self) -> PyResult<String> {
            Ok(self.issue.ident().clone())
        }

        #[getter]
        fn manual_label(&self) -> PyResult<Option<PyLabel>> {
            let obj = api2py_error(self.issue.get_manual_label())?
                .map(|label| PyLabel{inner: label});
            Ok(obj)
        }

        #[setter]
        fn set_manual_label(&self, value: PyLabel) -> PyResult<()> {
            api2py_error(self.issue.set_manual_label(value.inner))
        }

        fn invalidate_cached_label(&self) -> PyResult<()> {
            self.issue.invalidate_cached_label();
            Ok(())
        }

        #[getter]
        fn tags(&self) -> PyResult<Vec<String>> {
            api2py_error(self.issue.get_tags())
        }

        fn add_tag(&self, name: String) -> PyResult<()> {
            api2py_error(self.issue.add_tag(name))
        }

        fn remove_tag(&self, name: String) -> PyResult<()> {
            api2py_error(self.issue.remove_tag(name))
        }

        // fn start_review(&self) -> PyResult<()> {
        //     api2py_error(self.issue.start_review())
        // }
        //
        // fn finish_review(&self) -> PyResult<()> {
        //     api2py_error(self.issue.finish_review())
        // }

        #[getter]
        fn is_in_review(&self) -> PyResult<bool> {
            api2py_error(self.issue.in_review())
        }

        #[setter]
        fn set_is_in_review(&self, value: bool) -> PyResult<()> {
            let result = if value {
                self.issue.start_review()
            } else {
                self.issue.finish_review()
            };
            api2py_error(result)
        }

        #[getter]
        fn labelling_comments(&self) -> PyResult<Vec<PyComment>> {
            let comments = api2py_error(self.issue.get_labelling_comments())?
                .into_iter()
                .map(|c| PyComment{inner: c})
                .collect();
            Ok(comments)
        }

        pub fn add_labelling_comment(&self, text: String) -> PyResult<()> {
           api2py_error(self.issue.add_labelling_comment(text))
        }

        pub fn remove_labelling_comment(&self, comment: PyComment) -> PyResult<()> {
            api2py_error(self.issue.remove_labelling_comment(comment.inner))
        }

        #[getter]
        fn key(&self) -> PyResult<String> {
            api2py_error(self.issue.key().map(|x| x.clone()))
        }

        #[getter]
        fn summary(&self) -> PyResult<String> {
            api2py_error(self.issue.summary().map(|x| x.clone()))
        }

        #[getter]
        fn description(&self) -> PyResult<String> {
            api2py_error(self.issue.description().map(|x| x.clone()))
        }

        #[getter]
        fn comments(&self) -> PyResult<Vec<String>> {
            api2py_error(self.issue.comments().map(|x| x.clone()))
        }

        #[getter]
        fn status(&self) -> PyResult<String> {
            api2py_error(self.issue.status().map(|x| x.clone()))
        }

        #[getter]
        fn priority(&self) -> PyResult<String> {
            api2py_error(self.issue.priority().map(|x| x.clone()))
        }

        #[getter]
        fn resolution(&self) -> PyResult<Option<String>> {
            api2py_error(self.issue.resolution().map(|x| x.clone()))
        }

        #[getter]
        fn issue_type(&self) -> PyResult<String> {
            api2py_error(self.issue.issue_type().map(|x| x.clone()))
        }

        #[pyo3(signature=(*, attributes=vec![], load_labels=false))]
        fn issue_links(&self, attributes: Vec<String>, load_labels: bool) -> PyResult<Vec<PyIssue>> {
            let settings = IssueLoadingSettings::new(
                parse_issue_attributes(attributes)?, load_labels
            );
            let issues = api2py_error(self.issue.issue_links(settings))?
                .into_iter()
                .map(|i| PyIssue{issue: i})
                .collect();
            Ok(issues)
        }

        #[pyo3(signature=(*, attributes=vec![], load_labels=false))]
        fn parent(&self, attributes: Vec<String>, load_labels: bool) -> PyResult<Option<PyIssue>> {
            let settings = IssueLoadingSettings::new(
                parse_issue_attributes(attributes)?, load_labels
            );
            let issue = api2py_error(self.issue.parent(settings))?;
            Ok(issue.map(|i| PyIssue{issue: i}))
        }

        #[pyo3(signature=(*, attributes=vec![], load_labels=false))]
        fn subtasks(&self, attributes: Vec<String>, load_labels: bool) -> PyResult<Vec<PyIssue>> {
            let settings = IssueLoadingSettings::new(
                parse_issue_attributes(attributes)?, load_labels
            );
            let issues = api2py_error(self.issue.subtasks(settings))?
                .into_iter()
                .map(|i| PyIssue{issue: i})
                .collect();
            Ok(issues)
        }

        #[getter]
        fn watches(&self) -> PyResult<u64> {
            api2py_error(self.issue.watches())
        }

        #[getter]
        fn votes(&self) -> PyResult<u64> {
            api2py_error(self.issue.votes())
        }

        #[getter]
        fn date_created(&self) -> PyResult<String> {
            api2py_error(self.issue.date_created().map(|x| x.clone()))
        }

        #[getter]
        fn date_updated(&self) -> PyResult<String> {
            api2py_error(self.issue.date_updated().map(|x| x.clone()))
        }

        #[getter]
        fn date_resolved(&self) -> PyResult<Option<String>> {
            api2py_error(self.issue.date_resolved().map(|x| x.clone()))
        }

        #[getter]
        fn labels(&self) -> PyResult<Vec<String>> {
            api2py_error(self.issue.labels().map(|x| x.clone()))
        }

        #[getter]
        fn components(&self) -> PyResult<Vec<String>> {
            api2py_error(self.issue.components().map(|x| x.clone()))
        }

        #[getter]
        fn affected_versions(&self) -> PyResult<Vec<String>> {
            api2py_error(self.issue.affected_versions().map(|x| x.clone()))
        }

        #[getter]
        fn fix_versions(&self) -> PyResult<Vec<String>> {
            api2py_error(self.issue.fix_versions().map(|x| x.clone()))
        }
    }

    #[pyclass(name="Comment")]
    #[allow(unused)]
    #[derive(Debug, Clone)]
    struct PyComment {
        inner: Comment
    }

    #[pymethods]
    impl PyComment {
        fn __repr__(&self) -> PyResult<String> {
            Ok(format!("<Comment|id={}>", self.inner.identifier()))
        }

        #[getter]
        fn author(&self) -> PyResult<String> {
            Ok(self.inner.author().clone())
        }

        #[getter]
        fn body(&self) -> PyResult<String> {
            Ok(self.inner.text().clone())
        }

        #[setter]
        fn set_body(&mut self, text: String) -> PyResult<()> {
            api2py_error(self.inner.update_text(text))
        }
    }

    #[pyclass(name="Tag")]
    #[derive(Debug, PartialEq, Eq, Hash, Clone)]
    struct PyTag {
        inner: Tag
    }

    #[pymethods]
    impl PyTag {
        fn __repr__(&self) -> PyResult<String> {
            Ok(format!("<Tag|name={}>", self.inner.name()))
        }

        fn __richcmp__(&self, other: PyRef<PyTag>, op: CompareOp) -> Py<PyAny> {
            let py = other.py();
            match op {
                CompareOp::Eq => (self.inner == other.inner).into_py(py),
                CompareOp::Ne => (self.inner != other.inner).into_py(py),
                _ => py.NotImplemented(),
            }
        }

        fn __hash__(&self) -> PyResult<u64> {
            let mut s = std::collections::hash_map::DefaultHasher::new();
            self.inner.hash(&mut s);
            Ok(s.finish())
        }

        #[getter]
        fn name(&self) -> PyResult<String> {
            Ok(self.inner.name().clone())
        }

        #[getter]
        fn description(&self) -> PyResult<String> {
            Ok(self.inner.description().clone())
        }

        #[setter]
        fn set_description(&mut self, description: String) -> PyResult<()> {
            api2py_error(self.inner.update_description(description))
        }

        #[getter]
        fn tag_type(&self) -> PyResult<String> {
            let type_name = match self.inner.tag_type() {
                TagType::Author => "author",
                TagType::Project => "project",
                TagType::Custom => "custom"
            };
            Ok(type_name.to_string())
        }
    }

    #[pyclass(name="Embedding")]
    #[derive(Debug, PartialEq, Eq, Hash)]
    struct PyEmbedding {
        inner: Embedding
    }

    #[pymethods]
    impl PyEmbedding {
        fn __repr__(&self) -> PyResult<String> {
            Ok(format!("<Embedding|id={}>", self.inner.identifier()))
        }

        fn __richcmp__(&self, other: PyRef<PyEmbedding>, op: CompareOp) -> Py<PyAny> {
            let py = other.py();
            match op {
                CompareOp::Eq => (self.inner == other.inner).into_py(py),
                CompareOp::Ne => (self.inner != other.inner).into_py(py),
                _ => py.NotImplemented(),
            }
        }

        #[getter]
        fn identifier(&self) -> PyResult<String> {
            Ok(self.inner.identifier())
        }

        #[getter]
        fn name(&self) -> PyResult<String> {
            api2py_error(self.inner.name())
        }

        #[setter]
        fn set_name(&mut self, name: String) -> PyResult<()> {
            api2py_error(self.inner.update_name(name))
        }

        #[getter]
        fn config(&self, py: Python<'_>) -> PyResult<PyObject> {
            let mut config: HashMap<String, PyObject> = HashMap::new();
            for (k, v) in api2py_error(self.inner.config())? {
                config.insert(k.clone(), json_to_py(py, v.clone()));
            }
            Ok(config.into_py(py))
        }

        #[setter]
        fn set_config(&mut self, value: &PyAny) -> PyResult<()> {
            let json = py_to_json(value)?;
            if let Value::Object(c) = json {
                let config = c.into_iter().collect();
                api2py_error(self.inner.update_config(config))
            } else {
                Err(PyTypeError::new_err("Top-level JSON must be an object"))
            }

        }

        fn download_binary(&self, path: String) -> PyResult<()> {
            api2py_error(self.inner.download_binary(path))
        }

        fn upload_binary(&self, path: String) -> PyResult<()> {
            api2py_error(self.inner.upload_binary(path))
        }

        fn delete_binary(&self, ) -> PyResult<()> {
            api2py_error(self.inner.delete_binary())
        }
    }

    #[pyclass(name="Model")]
    #[allow(unused)]
    struct PyModel {
        inner: Model
    }

    #[pymethods]
    impl PyModel {
        fn __repr__(&self) -> PyResult<String> {
            Ok(format!("<Model|id={}>", self.inner.identifier()))
        }

        fn __richcmp__(&self, other: PyRef<PyModel>, op: CompareOp) -> Py<PyAny> {
            let py = other.py();
            match op {
                CompareOp::Eq => (self.inner == other.inner).into_py(py),
                CompareOp::Ne => (self.inner != other.inner).into_py(py),
                _ => py.NotImplemented(),
            }
        }

        #[getter]
        fn identifier(&self) -> PyResult<String> {
            Ok(self.inner.identifier())
        }

        #[getter]
        fn name(&self) -> PyResult<String> {
            api2py_error(self.inner.name())
        }

        #[setter]
        fn set_name(&mut self, name: String) -> PyResult<()> {
            api2py_error(self.inner.update_name(name))
        }

        #[getter]
        fn config(&self, py: Python<'_>) -> PyResult<PyObject> {
            let mut config: HashMap<String, PyObject> = HashMap::new();
            for (k, v) in api2py_error(self.inner.config())? {
                config.insert(k.clone(), json_to_py(py, v.clone()));
            }
            Ok(config.into_py(py))
        }

        #[setter]
        fn set_config(&mut self, value: &PyAny) -> PyResult<()> {
            let json = py_to_json(value)?;
            if let Value::Object(c) = json {
                let config = c.into_iter().collect();
                api2py_error(self.inner.update_config(config))
            } else {
                Err(PyTypeError::new_err("Top-level JSON must be an object"))
            }
        }

        #[getter]
        fn versions(&self) -> PyResult<Vec<PyVersion>> {
            let result = api2py_error(self.inner.model_versions())?;
            let converted = result.into_iter()
                .map(|v| PyVersion{inner: v})
                .collect();
            Ok(converted)
        }

        fn get_version_by_id(&self, id: String) -> PyResult<PyVersion> {
            let v = PyVersion{inner: api2py_error(self.inner.get_version_by_id(id))?};
            Ok(v)
        }

        fn add_version(&self, path: String, description: Option<String>) -> PyResult<PyVersion> {
            let version = api2py_error(self.inner.upload_version(path, description))?;
            Ok(PyVersion{inner: version})
        }

        fn remove_version(&self, version: &PyVersion) -> PyResult<()> {
            api2py_error(self.inner.delete_version(version.inner.clone()))
        }

        #[getter]
        fn test_runs(&self) -> PyResult<Vec<PyPerformance>> {
            let result = api2py_error(self.inner.model_runs())?;
            let converted = result.into_iter()
                .map(|r| PyPerformance{inner: r})
                .collect();
            Ok(converted)
        }

        fn get_run_by_id(&self, id: String) -> PyResult<PyPerformance> {
            let r = PyPerformance{inner: api2py_error(self.inner.get_run_by_id(id))?};
            Ok(r)
        }

        fn add_test_run(&self, data: Vec<&PyAny>, description: Option<String>) -> PyResult<PyPerformance> {
            let mut converted = Vec::with_capacity(data.len());
            for obj in data {
                converted.push(py_to_json(obj)?);
            }
            let run = api2py_error(self.inner.store_run(converted, description))?;
            Ok(PyPerformance{inner: run})
        }

        fn delete_test_run(&self, run: &PyPerformance) -> PyResult<()> {
            api2py_error(self.inner.delete_run(run.inner.clone()))
        }
    }

    #[pyclass(name="ModelVersion")]
    #[allow(unused)]
    struct PyVersion {
        inner: ModelVersion
    }

    #[pymethods]
    impl PyVersion {
        fn __repr__(&self) -> PyResult<String> {
            let text = format!("<Version|model_id={}, id={}>",
                               self.inner.model_id(),
                               self.inner.version_id());
            Ok(text)
        }

        fn __richcmp__(&self, other: PyRef<PyVersion>, op: CompareOp) -> Py<PyAny> {
            let py = other.py();
            match op {
                CompareOp::Eq => (self.inner == other.inner).into_py(py),
                CompareOp::Ne => (self.inner != other.inner).into_py(py),
                _ => py.NotImplemented(),
            }
        }

        #[getter]
        fn model_id(&self) -> PyResult<String> {
            Ok(self.inner.model_id())
        }

        #[getter]
        fn version_id(&self) -> PyResult<String> {
            Ok(self.inner.version_id())
        }

        fn download(&self, path: String) -> PyResult<()> {
            api2py_error(self.inner.download(path))
        }

        #[getter]
        fn predictions(&self, py: Python<'_>) -> PyResult<PyObject> {
            let pred = api2py_error(self.inner.get_predictions(None))?;
            let converted = pred
                .into_iter()
                .map(|(k, v)| (k, json_to_py(py, v)))
                .into_py_dict(py);
            Ok(converted.into_py(py))
        }

        #[setter]
        fn set_predictions(&self, predictions: HashMap<String, &PyAny>) -> PyResult<()> {
            let mut converted = HashMap::with_capacity(predictions.len());
            for (k, v) in predictions {
                converted.insert(k, py_to_json(v)?);
            }
            api2py_error(self.inner.store_predictions(converted))
        }

        fn delete_predictions(&self) -> PyResult<()> {
            api2py_error(self.inner.delete_predictions())
        }

        #[getter]
        pub fn description(&self) -> PyResult<String> {
            Ok(self.inner.description())
        }

        #[setter]
        pub fn set_description(&mut self, description: String) -> PyResult<()> {
            api2py_error(self.inner.update_description(description))
        }
    }

    #[pyclass(name="TestRun")]
    #[allow(unused)]
    struct PyPerformance {
        inner: TestRun
    }

    #[pymethods]
    impl PyPerformance {
        fn __repr__(&self) -> PyResult<String> {
            let text = format!(
                "<TestRun|model_id={}, id={}>", self.inner.model_id(), self.inner.run_id()
            );
            Ok(text)
        }

        #[getter]
        fn model_id(&self) -> PyResult<String> {
            Ok(self.inner.model_id())
        }

        #[getter]
        fn run_id(&self) -> PyResult<String> {
            Ok(self.inner.run_id())
        }

        #[getter]
        fn data(&self, py: Python<'_>) -> PyResult<Vec<PyObject>> {
            let result = api2py_error(self.inner.data())?;
            let mut converted = Vec::with_capacity(result.len());
            for fold in result {
                converted.push(json_to_py(py, fold));
            }
            Ok(converted)
        }

        #[getter]
        pub fn description(&self) -> PyResult<String> {
            Ok(self.inner.description())
        }

        #[setter]
        pub fn set_description(&mut self, description: String) -> PyResult<()> {
            api2py_error(self.inner.update_description(description))
        }
    }

    #[pymodule]
    fn issue_api(py: Python<'_>, m: &PyModule) -> PyResult<()> {
        m.add("IssueAPIError", py.get_type::<IssueAPIError>())?;
        m.add("InvalidCredentialsException", py.get_type::<InvalidCredentialsException>())?;
        m.add("NotAuthorizedException", py.get_type::<NotAuthorizedException>())?;
        m.add("InvalidTokenException", py.get_type::<InvalidTokenException>())?;
        m.add("HTTPException", py.get_type::<HTTPException>())?;
        m.add("LibraryException", py.get_type::<LibraryException>())?;
        m.add_class::<PyIssueRepository>()?;
        m.add_class::<PyIssue>()?;
        m.add_class::<PyQuery>()?;
        m.add_class::<PyLabel>()?;
        m.add_class::<PyRepo>()?;
        m.add_class::<PyTag>()?;
        m.add_class::<PyModel>()?;
        m.add_class::<PyVersion>()?;
        m.add_class::<PyPerformance>()?;
        m.add_class::<PyComment>()?;
        m.add_class::<PyEmbedding>()?;
        Ok(())
    }
}
