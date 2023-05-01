use std::fmt::{Display, Formatter};
use serde_json::{Map, Value};

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Query {
    Tag(QueryCMP, String),
    Project(String),
    Identifier(String),
    Key(String),
    And(Vec<Query>),
    Or(Vec<Query>),
    Exists(String, bool)
}


#[allow(unused)]
#[derive(Debug, Copy, Clone)]
pub enum QueryCMP { Eq, Ne }


#[allow(unused)]
impl Query {
    pub fn into_json(self) -> Value {
        match self {
            Query::Tag(cmp, tag) => {
                let mut map = Map::new();
                let mut inner_map = Map::new();
                let kw = match cmp {
                    QueryCMP::Eq => "$eq",
                    QueryCMP::Ne => "$ne"
                };
                inner_map.insert(kw.to_string(), Value::String(tag));
                map.insert("tags".to_string(), Value::Object(inner_map));
                Value::Object(map)
            }
            Query::Project(project) => {
                Query::Tag(QueryCMP::Eq, project).into_json()
            }
            Query::Identifier(ident) => {
                let mut map = Map::new();
                map.insert("_id".to_string(), Value::String(ident));
                Value::Object(map)
            }
            Query::Key(key) => {
                let mut map = Map::new();
                map.insert("key".to_string(), Value::String(key));
                Value::Object(map)
            }
            Query::And(branches) => {
                let mut map = Map::new();
                map.insert("$and".to_string(), serialize_logical_op_branches(branches));
                Value::Object(map)
            }
            Query::Or(branches) => {
                let mut map = Map::new();
                map.insert("$or".to_string(), serialize_logical_op_branches(branches));
                Value::Object(map)
            }
            Query::Exists(field, state) => {
                let mut map = Map::new();
                let mut inner_map = Map::new();
                inner_map.insert("$exists".to_string(), Value::Bool(state));
                map.insert(field, Value::Object(inner_map));
                Value::Object(map)
            }
        }
    }
}

fn serialize_logical_op_branches(arms: Vec<Query>) -> Value {
    let mut json_branches: Vec<Value> = Vec::with_capacity(arms.len());
    for branch in arms {
        json_branches.push(branch.into_json());
    }
    Value::Array(json_branches)
}


impl Display for Query {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Query::Tag(cmp, name) => match cmp {
                QueryCMP::Eq => write!(f, "{{\"tags\": {{\"$eq\": \"{}\"}}}}", name),
                QueryCMP::Ne => write!(f, "{{\"tags\": {{\"$ne\": \"{}\"}}}}", name)
            },
            Query::Project(project) => write!(f, "{{\"tags\": {{\"$eq\": \"{}\"}}}}", project),
            Query::Identifier(ident) => write!(f, "{{\"_id\": {{\"$eq\": \"{}\"}}}}", ident),
            Query::Key(key) => write!(f, "{{\"key\": {{\"$eq\": \"{}\"}}}}", key),
            Query::And(arms)=> {
                write!(f, "{{\"$and\": [")?;
                write_logical_op_body(f, arms)?;
                write!(f, "]}}")?;
                Ok(())
            },
            Query::Or(arms) => {
                write!(f, "{{\"$or\": [")?;
                write_logical_op_body(f, arms)?;
                write!(f, "]}}")?;
                Ok(())
            },
            Query::Exists(value, state) => {
                write!(f, "\"{}\": {{\"$exists\": {}}}}}", value, state)
            }
        }
    }
}

fn write_logical_op_body(f: &mut Formatter<'_>, arms: &Vec<Query>) -> std::fmt::Result {
    let mut first = true;
    for a in arms {
        if !first {
            write!(f, ", {}", a)?;
        } else {
            write!(f, "{}", a)?;
            first = false;
        }
    }
    Ok(())
}
