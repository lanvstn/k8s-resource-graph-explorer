// jsonnet examples/edge_query.jsonnet | curl -H 'Content-Type: application/json' -d@- http://127.0.0.1:3000/v1/query/edges | jq

// resource {
//    api: String,
//    kind: String,
//    namespace: String,
//    name: String,
//    =>
//    obj: Json
//}

{
    "query_string": |||
    
    # Find ownership edges 
    # TODO: This rule is horribly slow even for a base k8s cluster (5 seconds)
    #       Compare that to quering the whole resource relation which is like 10ms.
    #       I'm definitely doing something stupid here, CozoDB should be faster.
    #       So yeah this is not the best query but it should demonstrate that it at least works

    edge[from, to, label] := 
        *resource{api: f_api, kind: f_kind, namespace: f_namespace, name: f_name, obj: f_obj}, 
        *resource{api: t_api, kind: t_kind, namespace: t_namespace, name: t_name, obj: t_obj}, 

        f_namespace = t_namespace, # resolve ownership within the same namespace

        idx in [0, 1, 2], # TODO: fix whatever made me write this horrible hack
        owner = t_obj -> ['metadata', 'ownerReferences', idx],
        owner_api = owner -> 'apiVersion', owner_api = f_api,
        owner_kind = owner -> 'kind', owner_kind = f_kind,
        owner_name = owner -> 'name', owner_name = f_name,
        
        from = [f_api, f_kind, f_namespace, f_name],
        to = [t_api, t_kind, t_namespace, t_name],
        label = 'owns'

    ?[from, to, label] := edge[from, to, label]
        
|||
}
