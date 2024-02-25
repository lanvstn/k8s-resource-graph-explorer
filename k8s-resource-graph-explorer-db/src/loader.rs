//! Load resources from Kubernetes into CozoDB

use std::collections::BTreeMap;

use cozo::{DataValue, DbInstance, ScriptMutability};
use kube::{api::DynamicObject, discovery::verbs, Api, Client, Discovery, ResourceExt};
use smartstring::alias::String;
use tracing::debug;

use crate::resource;

// TODO: rather than a one-off sync this should manage set of dynamic watchers in the background
pub async fn sync_cluster(db: &DbInstance) -> anyhow::Result<()> {
    let client = Client::try_default().await?;

    let discovery = Discovery::new(client.clone()).run().await?;
    for group in discovery.groups() {
        for (ar, caps) in group.recommended_resources() {
            if !caps.supports_operation(verbs::LIST) {
                continue;
            }

            let api: Api<DynamicObject> = Api::all_with(client.clone(), &ar);

            debug!("discovered {}/{} {}", group.name(), ar.version, ar.kind);

            let list = api.list(&Default::default()).await?;
            let to_put: Vec<DataValue> = list
                .items
                .iter()
                .map(move |item| {
                    resource::Resource {
                        api: String::from(String::from(group.name()) + "/" + &ar.version),
                        kind: String::from(&ar.kind),
                        namespace: item // Option<String> to Option<SmartString> conversion
                            .namespace()
                            .clone()
                            .and_then(|ns| Some(String::from(ns))),
                        name: String::from(&item.name_any()),
                        obj: Some(serde_json::to_value(item.clone()).unwrap()),
                    }
                    .as_cozo_full()
                })
                .collect();

            let result = db.run_script(
                "
?[api, kind, namespace, name, obj] <- $data
:put resource {api, kind, namespace, name, obj}",
                BTreeMap::from([("data".to_owned(), DataValue::List(to_put))]),
                ScriptMutability::Mutable,
            );
            debug!("{:?}", result);
        }
    }

    Ok(())
}
