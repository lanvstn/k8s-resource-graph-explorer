//! Resource definition and interactions

use anyhow::anyhow;
use cozo::{DataValue, DbInstance, JsonData, ScriptMutability};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use smartstring::alias::String;
use thiserror::Error;
use tracing::{debug, info};

#[derive(Error, Debug)]
pub enum ResourceError {
    #[error("unexpected type for column {0:?}")]
    Type(ColumnType),

    #[error("not enough columns in result: tried to get column index {0}")]
    ColumnCount(usize),
}

#[derive(Debug)]
pub enum ColumnType {
    String,
    Json,
}

#[derive(Serialize, Deserialize)]
pub struct Resource {
    pub api: String,
    pub kind: String,
    pub namespace: Option<String>,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub obj: Option<Value>,
}

impl Resource {
    pub fn as_cozo_key(&self) -> DataValue {
        DataValue::List(vec![
            DataValue::Str(self.api.clone()),
            DataValue::Str(self.kind.clone()),
            DataValue::Str(self.namespace.clone().unwrap_or_default()),
            DataValue::Str(self.name.clone()),
        ])
    }

    pub fn as_cozo_full(&self) -> DataValue {
        DataValue::List(vec![
            DataValue::Str(self.api.clone()),
            DataValue::Str(self.kind.clone()),
            DataValue::Str(self.namespace.clone().unwrap_or_default()),
            DataValue::Str(self.name.clone()),
            DataValue::Json(JsonData(self.obj.clone().unwrap_or_default())),
        ])
    }

    // TODO: json is currently never retrieved from cozo (no need yet)
    pub fn from_cozo(row: Vec<DataValue>) -> anyhow::Result<Self> {
        // String => Some(String) if ""
        let maybe_string = |s: String| (!s.is_empty()).then_some(s);
        let row_string = make_row_string(&row);

        Ok(Resource {
            api: row_string(0)?,
            kind: row_string(1)?,
            namespace: maybe_string(row_string(2)?),
            name: row_string(3)?,
            obj: None,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct Edge {
    from: Resource,
    to: Resource,
    label: String,
}

impl Edge {
    pub fn from_cozo(row: Vec<DataValue>) -> anyhow::Result<Self> {
        let row_resource = |index: usize| -> anyhow::Result<Resource> {
            let resource_value = row
                .get(index)
                .ok_or(ResourceError::ColumnCount(index))?
                .get_slice()
                .ok_or(ResourceError::Type(ColumnType::String))?
                .to_vec();
            Ok(Resource::from_cozo(resource_value)?)
        };

        Ok(Edge {
            from: row_resource(0)?,
            to: row_resource(1)?,
            label: make_row_string(&row)(2)?,
        })
    }
}

// Closure that returns the string at the given index in row
// TODO: use named rows instead of using assumed indexes?
fn make_row_string<'a>(row: &'a Vec<DataValue>) -> impl Fn(usize) -> anyhow::Result<String> + 'a {
    |index: usize| -> anyhow::Result<String> {
        let s = row
            .get(index)
            .ok_or(ResourceError::ColumnCount(index))?
            .get_str()
            .ok_or(ResourceError::Type(ColumnType::String))?;
        Ok(String::from(s))
    }
}

pub fn res_query(db: &DbInstance, query_string: &str) -> anyhow::Result<Vec<Resource>> {
    debug!("query string: {:?}", query_string);
    info!("running resource query");
    let result = db.run_script(
        query_string,
        Default::default(),
        ScriptMutability::Immutable,
    );
    info!("finished resource query");

    let result = match result {
        Ok(result) => Ok(result),
        Err(e) => Err(anyhow!("query error: {}", e)),
    }?;

    debug!("resource query result: {:?}", result);

    result
        .into_iter()
        .map(|row| Resource::from_cozo(row))
        .collect()
}

pub fn edge_query(db: &DbInstance, query_string: &str) -> anyhow::Result<Vec<Edge>> {
    // TODO add params
    debug!("query string: {:?}", query_string);
    info!("running edge query"); // this is so slow I have to log it so you know it's not hanging
    let result = db.run_script(
        query_string,
        Default::default(),
        ScriptMutability::Immutable,
    );
    info!("finished edge query");

    // TODO: figure out why anyhow doesn't work with the Cozo error.
    // Then this match should be unneeded...
    let result = match result {
        Ok(result) => Ok(result),
        Err(e) => Err(anyhow!("query error: {}", e)),
    }?;

    debug!("edge query result: {:?}", result);

    // TODO: return the iterator instead of collecting it?
    result.into_iter().map(|row| Edge::from_cozo(row)).collect()
}
