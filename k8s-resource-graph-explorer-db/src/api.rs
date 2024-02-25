//! Everything directly related to serving the query API.

use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use cozo::DbInstance;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::info;

use crate::resource;

// TODO: implement a proper error type for API calls so we can return something other than 500
struct APIError(anyhow::Error);

impl IntoResponse for APIError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for APIError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

struct APIState {
    db: DbInstance,
}

#[derive(Serialize, Deserialize)]
struct SimpleQuery {
    query_string: String,
}

#[derive(Serialize, Deserialize)]
struct EdgeQuery {
    // TODO: include "currently selected" resources as params to constrain what edges are queried
    //active_resources: Vec<resource::Resource>,
    query_string: String,
}

#[derive(Serialize, Deserialize)]
struct ResourceResponse {
    resources: Vec<resource::Resource>,
}

#[derive(Serialize, Deserialize)]
struct EdgeResponse {
    edges: Vec<resource::Edge>,
}

async fn res_query(
    State(state): State<Arc<APIState>>,
    Json(payload): Json<Value>,
) -> Result<Json<ResourceResponse>, APIError> {
    let dbq: SimpleQuery = serde_json::from_value(payload)?;
    let resources = resource::res_query(&state.db, dbq.query_string.as_str())?;

    Ok(Json(ResourceResponse { resources }))
}

async fn edge_query(
    State(state): State<Arc<APIState>>,
    Json(payload): Json<Value>,
) -> Result<Json<EdgeResponse>, APIError> {
    let dbq: EdgeQuery = serde_json::from_value(payload)?;
    let edges = resource::edge_query(&state.db, dbq.query_string.as_str())?;

    Ok(Json(EdgeResponse { edges }))
}

pub async fn serve(db: DbInstance) -> anyhow::Result<()> {
    let shared_state = Arc::new(APIState { db: db });

    let app = Router::new()
        .route("/", get(|| async { "?[] <- [['ヾ(≧▽≦*)o']]\n" }))
        .route("/v1/query/resources", post(res_query))
        .route("/v1/query/edges", post(edge_query))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("listening on 0.0.0.0:3000");
    axum::serve(listener, app).await?;

    Ok(())
}
