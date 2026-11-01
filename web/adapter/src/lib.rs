//! A bounded consumer of existing RLINE graph kernels. No shared API changes.
use rgraph_core::{DirectedWeightedGraph, WeightedEdge};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const MAX_REQUEST: usize = 131_072;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Request {
    nodes: Vec<String>,
    edges: Vec<Edge>,
    source: u32,
    #[serde(default)]
    selected: Vec<u32>,
    #[serde(default)]
    blocked_edges: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Edge {
    id: String,
    from: u32,
    to: u32,
    weight: f64,
}

struct Graph<'a>(&'a Request);
impl DirectedWeightedGraph for Graph<'_> {
    type EdgeId = usize;
    fn node_count(&self) -> usize {
        self.0.nodes.len()
    }
    fn outgoing_edges(&self, source: usize) -> Vec<WeightedEdge<usize>> {
        let mut out = Vec::new();
        for (id, edge) in self.0.edges.iter().enumerate() {
            if edge.from as usize == source {
                out.push(WeightedEdge {
                    id,
                    target: edge.to as usize,
                    weight: edge.weight,
                });
            }
            if edge.to as usize == source && edge.from != edge.to {
                out.push(WeightedEdge {
                    id,
                    target: edge.from as usize,
                    weight: edge.weight,
                });
            }
        }
        out
    }
}

fn inspect(text: &str) -> Result<Value, String> {
    if text.len() > MAX_REQUEST {
        return Err("RLINE-WEB-LIMIT input exceeds 128 KiB UTF-8".into());
    }
    let req: Request = serde_json::from_str(text).map_err(|e| format!("RLINE-WEB-JSON {e}"))?;
    let n = req.nodes.len();
    if !(1..=64).contains(&n) || req.edges.len() > 512 {
        return Err("RLINE-WEB-LIMIT use 1–64 nodes and at most 512 edges".into());
    }
    if req
        .nodes
        .iter()
        .any(|label| label.trim().is_empty() || label.len() > 128)
    {
        return Err("RLINE-WEB-LABEL node labels need 1–128 UTF-8 bytes".into());
    }
    if req.source as usize >= n {
        return Err("RLINE-WEB-SOURCE source node is out of bounds".into());
    }
    let mut ids = BTreeSet::new();
    for edge in &req.edges {
        if edge.id.trim().is_empty() || edge.id.len() > 64 || !ids.insert(edge.id.as_str()) {
            return Err("RLINE-WEB-ID edge IDs need 1–64 UTF-8 bytes and must be unique".into());
        }
        if edge.from as usize >= n || edge.to as usize >= n {
            return Err(format!(
                "RLINE-WEB-ENDPOINT edge {} has an out-of-bounds endpoint",
                edge.id
            ));
        }
        if !edge.weight.is_finite() || edge.weight < 0.0 || edge.weight > 1e100 {
            return Err(format!(
                "RLINE-WEB-WEIGHT edge {} needs a finite weight in [0, 1e100]",
                edge.id
            ));
        }
    }
    let selected: BTreeSet<_> = req.selected.iter().copied().collect();
    if selected.len() != req.selected.len() || req.selected.iter().any(|&node| node as usize >= n) {
        return Err("RLINE-WEB-SELECTION selected nodes must be unique valid indices".into());
    }
    let blocked: BTreeSet<_> = req.blocked_edges.iter().map(String::as_str).collect();
    if blocked.len() != req.blocked_edges.len() || blocked.iter().any(|id| !ids.contains(id)) {
        return Err("RLINE-WEB-BLOCKED blocked edge IDs must be unique and declared".into());
    }
    let graph = Graph(&req);
    let active = |id: usize| !blocked.contains(req.edges[id].id.as_str());
    let tree =
        rgraph_core::single_source_shortest_paths_with_filter(&graph, req.source as usize, active)
            .map_err(|e| format!("RLINE-NATIVE-PATH {e}"))?;
    let reachable = rgraph_core::reachable_nodes_with_filter(&graph, req.source as usize, active)
        .map_err(|e| format!("RLINE-NATIVE-REACHABLE {e}"))?;
    let components = rgraph_core::connected_components_with_filter(&graph, active)
        .map_err(|e| format!("RLINE-NATIVE-COMPONENTS {e}"))?;
    let selected_indices: Vec<_> = req.selected.iter().map(|&i| i as usize).collect();
    let selected_components =
        rgraph_core::connected_components_in_nodes_with_filter(&graph, &selected_indices, active)
            .map_err(|e| format!("RLINE-NATIVE-SELECTION {e}"))?;
    let bridges = rgraph_core::bridges_with_filter(&graph, active)
        .map_err(|e| format!("RLINE-NATIVE-BRIDGES {e}"))?;
    let articulation = rgraph_core::articulation_points_with_filter(&graph, active)
        .map_err(|e| format!("RLINE-NATIVE-ARTICULATION {e}"))?;
    let mut adjacency = vec![Vec::<usize>::new(); n];
    for edge in req
        .edges
        .iter()
        .filter(|e| !blocked.contains(e.id.as_str()))
    {
        adjacency[edge.from as usize].push(edge.to as usize);
        if edge.from != edge.to {
            adjacency[edge.to as usize].push(edge.from as usize);
        }
    }
    let selection: Vec<_> = (0..n).map(|i| selected.contains(&(i as u32))).collect();
    let metrics = rgraph_core::undirected_boundary_metrics(&adjacency, &selection)
        .map_err(|e| format!("RLINE-NATIVE-BOUNDARY {e}"))?;
    let bridge_ids: BTreeSet<_> = bridges
        .iter()
        .map(|b| req.edges[b.edge_id].id.as_str())
        .collect();
    let bridge_records: Vec<_> = bridges
        .iter()
        .map(|b| json!({"source":b.source,"target":b.target,"edge_id":req.edges[b.edge_id].id}))
        .collect();
    let boundary = json!({"selected_count":metrics.selected_count,"complement_count":metrics.complement_count,"selected_internal_edges":metrics.selected_internal_edges,"complement_internal_edges":metrics.complement_internal_edges,"boundary_edges":metrics.boundary_edges,"selected_degree":metrics.selected_degree,"complement_degree":metrics.complement_degree,"total_edges":metrics.total_edges,"conductance":metrics.conductance});
    let predecessors: Vec<_> = tree
        .predecessors
        .iter()
        .map(|group| {
            group
                .iter()
                .map(|p| json!({"node":p.node,"edge_id":req.edges[p.edge_id].id}))
                .collect::<Vec<_>>()
        })
        .collect();
    Ok(
        json!({"schema":"rline.graph-workbench.v1","ok":true,"request_json":text,"graph":req,
        "distances":tree.distances,"reachable":reachable,"components":components,"selected_components":selected_components,
        "bridge_records":bridge_records,"bridge_edge_ids":bridge_ids,"articulation_points":articulation,"boundary":boundary,
        "predecessors":predecessors,"native_path_counts":tree.path_counts,"visit_order":tree.visit_order,
        "boundary_json":serde_json::to_string_pretty(&boundary).map_err(|e| e.to_string())?,
        "limits":{"nodes":64,"edges":512,"input_utf8_bytes":MAX_REQUEST,"maximum_weight":"1e100","native_absolute_distance_tolerance":"1e-9"},
        "notes":["Undirected abstract graph. Edge weights have caller-defined units; no geographic, service, election or fairness claim.","Distances use existing native f64/absolute 1e-9 comparisons. Null means unreachable. Tiny cost differences can be treated as ties.","Topology metrics ignore weights and blocked edges. Boundary metrics collapse parallel node pairs and ignore loops. Zero-volume conductance is native 0, not evidence of a good partition.","Bridge records retain native oriented incidences; bridge_edge_ids deduplicates their external IDs. Parallel edges prevent a single-edge bridge.","Native path counts/predecessors are diagnostics, not certified path enumeration: zero-cost cycles and tolerance ties can make them unsuitable as multiplicity evidence.","This consumer changes no shared kernel API, family manifest, algebra plan, RCTX/RHIST contract or downstream adoption status."]}),
    )
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub fn replay(text: &str) -> String {
    match inspect(text) {
        Ok(value) => value.to_string(),
        Err(message) => {
            json!({"schema":"rline.graph-workbench.v1","ok":false,"diagnostic":{"message":message}})
                .to_string()
        }
    }
}

pub fn examples() -> String {
    let base = json!({"nodes":["North","East","South","West","Isolated"],"edges":[{"id":"north-east","from":0,"to":1,"weight":2.0},{"id":"east-south","from":1,"to":2,"weight":3.0},{"id":"north-south","from":0,"to":2,"weight":7.0},{"id":"south-west","from":2,"to":3,"weight":1.0}],"source":0,"selected":[0,1],"blocked_edges":[]});
    let mut blocked = base.clone();
    blocked["blocked_edges"] = json!(["south-west"]);
    let mut parallel = base.clone();
    parallel["edges"].as_array_mut().unwrap().extend([
        json!({"id":"parallel","from":2,"to":3,"weight":2.0}),
        json!({"id":"loop","from":0,"to":0,"weight":0.0}),
    ]);
    let mut zero = base.clone();
    for edge in zero["edges"].as_array_mut().unwrap() {
        edge["weight"] = json!(0.0);
    }
    let mut empty_selection = base.clone();
    empty_selection["selected"] = json!([]);
    json!({"graph":base,"blocked":blocked,"parallel":parallel,"zero-cost":zero,"empty-selection":empty_selection,"single-node":{"nodes":["Only node"],"edges":[],"source":0,"selected":[0],"blocked_edges":[]}}).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request(name: &str) -> Value {
        serde_json::from_str::<Value>(&examples()).unwrap()[name].clone()
    }
    fn result(req: Value) -> Value {
        serde_json::from_str(&replay(&req.to_string())).unwrap()
    }
    #[test]
    fn examples_have_native_results() {
        for (_, req) in serde_json::from_str::<Value>(&examples())
            .unwrap()
            .as_object()
            .unwrap()
        {
            assert_eq!(result(req.clone())["ok"], true);
        }
    }
    #[test]
    fn known_distances_components_and_cut() {
        let r = result(request("graph"));
        assert_eq!(r["distances"], json!([0.0, 2.0, 5.0, 6.0, null]));
        assert_eq!(r["components"], json!([[0, 1, 2, 3], [4]]));
        assert_eq!(r["boundary"]["boundary_edges"], 2);
        assert_eq!(r["bridge_edge_ids"], json!(["south-west"]));
        assert_eq!(r["articulation_points"], json!([2]));
    }
    #[test]
    fn blocking_changes_reachability_without_rewriting_input() {
        let req = request("blocked");
        let raw = serde_json::to_string_pretty(&req)
            .unwrap()
            .replace('\n', "\r\n");
        let r: Value = serde_json::from_str(&replay(&raw)).unwrap();
        assert_eq!(r["request_json"], raw);
        assert_eq!(r["distances"][3], Value::Null);
        assert_eq!(r["components"].as_array().unwrap().len(), 3);
    }
    #[test]
    fn parallel_edges_remove_bridge_and_loops_leave_cut_unchanged() {
        let r = result(request("parallel"));
        assert_eq!(r["bridge_edge_ids"], json!([]));
        assert_eq!(r["boundary"], result(request("graph"))["boundary"]);
    }
    #[test]
    fn empty_selection_zero_volume_is_explicit() {
        let r = result(request("empty-selection"));
        assert_eq!(r["selected_components"], json!([]));
        assert_eq!(r["boundary"]["conductance"], 0.0);
    }
    #[test]
    fn invalid_and_unknown_input_fail() {
        let mut req = request("graph");
        req["extra"] = json!(42);
        assert_eq!(result(req)["ok"], false);
        let mut req = request("graph");
        req["edges"][0]["weight"] = json!(-1);
        assert!(result(req)["diagnostic"]["message"]
            .as_str()
            .unwrap()
            .contains("WEIGHT"));
    }
    #[test]
    fn duplicate_ids_and_bad_endpoints_fail() {
        let mut req = request("graph");
        req["edges"][1]["id"] = req["edges"][0]["id"].clone();
        assert_eq!(result(req)["ok"], false);
        let mut req = request("graph");
        req["edges"][0]["to"] = json!(99);
        assert_eq!(result(req)["ok"], false);
    }
    #[test]
    fn byte_limit_and_portable_index_fail_closed() {
        assert!(replay(&" ".repeat(MAX_REQUEST + 1)).contains("LIMIT"));
        let mut req = request("graph");
        req["source"] = json!(4294967296_u64);
        assert_eq!(result(req)["ok"], false);
    }
}
