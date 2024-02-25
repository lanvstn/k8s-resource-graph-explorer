use cozo::{DbInstance, ScriptMutability};
use tracing::*;

pub mod api;
pub mod loader;
pub mod resource;

fn init_db(db: &DbInstance) {
    // I have a feeling the JSON type is ruining performance, maybe I should expand obj to
    // path => value rows which will make loading and updating slower but might completely fix query perf.
    let init_script = ":create resource {
    api: String,
    kind: String,
    namespace: String,
    name: String,
    =>
    obj: Json
}";
    let result = db
        .run_script(init_script, Default::default(), ScriptMutability::Mutable)
        .unwrap();
    debug!("{:?}", result);

    // test query: ?[api, kind, namespace, name, obj] := *resource{api, kind, namespace, name, obj}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let db = DbInstance::new("mem", "", Default::default()).unwrap();

    init_db(&db);

    info!("syncing cluster to local DB...");
    loader::sync_cluster(&db).await?;

    api::serve(db).await
}
