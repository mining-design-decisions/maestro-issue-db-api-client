use serde_json::Value;

fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where T: serde::Deserialize<'de>,
          D: serde::Deserializer<'de>
{
    serde::Deserialize::deserialize(deserializer).map(Some)
}



#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct RawPriority {
    #[serde(rename(deserialize = "self"))] pub self_: String,
    #[serde(rename(deserialize = "iconUrl"))] pub icon_url: String,
    pub name: String,
    pub id: String
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct RawVersion {
    #[serde(rename(deserialize = "self"))] pub self_: String,
    pub name: String,
    pub id: String,
    pub archived: bool,
    pub released: bool,
    #[serde(rename(deserialize = "releaseDate"))] pub release_date: String
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct RawStatus {
    #[serde(rename(deserialize = "self"))] pub self_: String,
    #[serde(rename(deserialize = "iconUrl"))] pub icon_url: String,
    pub description: String,
    pub name: String,
    pub id: String,
    #[serde(rename(deserialize = "statusCategory"))] pub category: Value
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct RawComponent {
    #[serde(rename(deserialize = "self"))] pub self_: String,
    pub id: String,
    pub name: String,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct RawVotes {
    #[serde(rename(deserialize = "self"))] pub self_: String,
    pub votes: u64,
    #[serde(rename(deserialize = "hasVoted"))] pub has_voted: bool
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct RawIssueType {
    #[serde(rename(deserialize = "self"))] pub self_: String,
    pub id: String,
    pub description: String,
    #[serde(rename(deserialize = "iconUrl"))] pub icon_url: String,
    pub name: String,
    pub subtask: bool,
    #[serde(rename(deserialize = "avatarId"))] pub avatar_id: u64
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct RawWatches {
    #[serde(rename(deserialize = "self"))] pub self_: String,
    #[serde(rename(deserialize = "watchCount"))] pub watch_count: u64,
    #[serde(rename(deserialize = "isWatching"))] pub is_watching: bool
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct RawResolution {
    #[serde(rename(deserialize = "self"))] pub self_: String,
    pub id: String,
    pub description: String,
    pub name: String
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct RawIssueLinkDescription {
    pub id: String,
    pub name: String,
    pub inward: String,
    pub outward: String,
    #[serde(rename(deserialize = "self"))] pub self_: String,
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub struct RawComment {
    #[serde(rename(deserialize = "self"))] pub self_: String,
    pub id: String,
    pub author: Value,
    #[serde(rename(deserialize = "updateAuthor"))] pub update_author: Value,
    pub created: String,
    pub updated: String,
    pub body: String
}

#[allow(unused)]
#[derive(Debug, serde::Deserialize)]
pub enum RawIssueLink {
    InwardIssue{
        #[serde(rename(deserialize = "self"))] self_: String,
        id: String,
        #[serde(rename(deserialize = "type"))] type_: RawIssueLinkDescription,
        #[serde(rename(deserialize = "inwardIssue"))] inward_issue: String
    },
    OutWardIssue{
        #[serde(rename(deserialize = "self"))] self_: String,
        id: String,
        #[serde(rename(deserialize = "type"))] type_: RawIssueLinkDescription,
        #[serde(rename(deserialize = "outwardIssue"))] outward_issue: String
    }
}


#[allow(unused)]
#[derive(Debug, serde::Deserialize, Default)]
pub struct RawIssueData {
    #[serde(default)] pub key: Option<String>,
    #[serde(default)] pub summary: Option<String>,
    #[serde(default)] pub description: Option<String>,
    #[serde(default)] pub comments: Option<Vec<RawComment>>,
    #[serde(default)] pub status: Option<RawStatus>,
    #[serde(default)] pub priority: Option<RawPriority>,
    #[serde(default, deserialize_with = "deserialize_some")] pub resolution: Option<Option<RawResolution>>,
    #[serde(default)] pub issuetype: Option<RawIssueType>,
    #[serde(default)] pub issuelinks: Option<Vec<RawIssueLink>>,
    #[serde(default, deserialize_with = "deserialize_some")] pub parent: Option<Option<String>>,
    #[serde(default)] pub subtasks: Option<Vec<String>>,
    #[serde(default)] pub watches: Option<RawWatches>,
    #[serde(default)] pub votes: Option<RawVotes>,
    #[serde(default)] pub created: Option<String>,
    #[serde(default)] pub updated: Option<String>,
    #[serde(default, deserialize_with = "deserialize_some")] pub resolutiondate: Option<Option<String>>,
    #[serde(default)] pub labels: Option<Vec<String>>,
    #[serde(default)] pub components: Option<Vec<RawComponent>>,
    #[serde(default)] pub versions: Option<Vec<RawVersion>>,
    #[serde(default, rename(deserialize = "fixVersions"))] pub fix_versions: Option<Vec<RawVersion>>
}
