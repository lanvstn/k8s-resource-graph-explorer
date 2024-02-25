# k8s-resource-graph-explorer

Explore your Kubernetes cluster with the power of Datalog, using [CozoDB](https://www.cozodb.org/).

I made this because I wanted to get into Datalog, and to get better at Rust.
It's far from practically usable but the basic idea works. 

How this works:

- It loads your entire Kubernetes cluster state into an in-memory CozoDB instance
- Then it exposes a query API to build a UI against

Currently this repo contains only the backend part but the idea is to make a nice graph UI over this one day
where you can write queries and see the result. 
On top of that, the UI could abstract away the raw queries and provide a mode where you can explore the cluster
by starting with a specific resource and then expanding its relations. 

Imagine some preset queries could be provided to analyze the cluster for potential issues: 
you could query for all configmaps that are not referenced by any other resources for example.

I believe there's a lot of potential hidden here: 
having a nice query language for the cluster state could be super useful for building debug tools or teaching Kubernetes concepts.

## Demo

```bash
jsonnet examples/edge_query.jsonnet | curl -H 'Content-Type: application/json' -d@- http://127.0.0.1:3000/v1/query/edges | jq
```

(result additionally filtered to coredns for brevity)

```json
{
  "edges": [
    {
      "from": {
        "api": "apps/v1",
        "kind": "Deployment",
        "namespace": "kube-system",
        "name": "coredns"
      },
      "to": {
        "api": "apps/v1",
        "kind": "ReplicaSet",
        "namespace": "kube-system",
        "name": "coredns-5d78c9869d"
      },
      "label": "owns"
    },
    {
      "from": {
        "api": "apps/v1",
        "kind": "ReplicaSet",
        "namespace": "kube-system",
        "name": "coredns-5d78c9869d"
      },
      "to": {
        "api": "/v1",
        "kind": "Pod",
        "namespace": "kube-system",
        "name": "coredns-5d78c9869d-r5zk2"
      },
      "label": "owns"
    },
    {
      "from": {
        "api": "apps/v1",
        "kind": "ReplicaSet",
        "namespace": "kube-system",
        "name": "coredns-5d78c9869d"
      },
      "to": {
        "api": "/v1",
        "kind": "Pod",
        "namespace": "kube-system",
        "name": "coredns-5d78c9869d-zpwl4"
      },
      "label": "owns"
    },
  ]
}
```

## Getting started

// TODO

There's no getting started guide because I can't recommend anyone to "get started" with this at all right now.

Please beware that when you start k8s-resource-graph-explorer-db it will immediately try to download your entire cluster into an in-memory database.

## Query API

k8s-resource-graph-explorer implements CozoDB, so this readme assumes you have read 
https://docs.cozodb.org/en/latest/tutorial.html along with the rest of the documentation.

The database contains all Kubernetes resources using the following relation:

```
resource {
    api: String,
    kind: String,
    namespace: String,
    name: String,
    =>
    obj: Json
}
```

### `/v1/query/resources`

Expected result format: `?[api, kind, namespace, name]`.

There's an example in [simple_query.json](./k8s-resource-graph-explorer-db/simple_query.json)

### `/v1/query/edges`

Expected result format: `?[from, to, label]` where from and to are resources (see resource query API) and label is a string.

There's an example in [edge_query.jsonnet](./k8s-resource-graph-explorer-db/edge_query.jsonnet).

TODO: add query param with a list of `resource`s to reference inside the query string for filtering.

## Security model

This may be improved in the future, but the current security model is
that anyone who can access this application has FULL read access to your cluster.

It's therefore recommended to only run this 

- as a local tool from your local machine (where it will have the same permissions as you)
- as a service on development clusters where this kind of full read access is no issue

There are no features that can modify your cluster.

## Future work

- Fix the query performance (I did something wrong there but I think I know what already)
- UI
- Live reload of cluster state
- Implement time travel (what related resources existed yesterday but not today?)
- Add source map support (associate resources with source code like Helm charts, Jsonnet files?)
- NetworkPolicy analysis tooling (e.g. which pods can/cannot access my pod)
- Come up with a better name
