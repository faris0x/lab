use mathcore::Rat;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::unionfind::UnionFind;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Deserialize)]
struct GraphInput {
    labels: Vec<String>,
    directed: bool,
    weights: Vec<Vec<Option<String>>>,
    algorithm: String,
    #[serde(default)]
    source: usize,
    #[serde(default)]
    target: usize,
}

#[derive(Clone, Serialize)]
struct Step {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    latex: Option<String>,
}

#[derive(Serialize)]
struct Output {
    ok: bool,
    title: String,
    summary: Vec<String>,
    steps: Vec<Step>,
    latex: String,
}

#[derive(Clone, Copy)]
struct Edge {
    u: usize,
    v: usize,
    w: Rat,
}

#[derive(PartialEq, Eq)]
struct HeapEntry {
    cost: Rat,
    node: usize,
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .cost
            .cmp_rat(self.cost)
            .then_with(|| other.node.cmp(&self.node))
    }
}

impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn label(labels: &[String], index: usize) -> String {
    labels
        .get(index)
        .cloned()
        .unwrap_or_else(|| index.to_string())
}

fn text(value: &str) -> String {
    format!(
        "\\text{{{}}}",
        value.replace('\\', "\\textbackslash{}").replace('&', "\\&")
    )
}

fn math(value: &str) -> String {
    format!("${}$", value)
}

fn weight(value: Rat) -> String {
    value.latex()
}

fn rat_lt(a: Rat, b: Rat) -> bool {
    a.cmp_rat(b) == Ordering::Less
}

fn parse_edges(input: &GraphInput) -> Result<Vec<Edge>, String> {
    let n = input.labels.len();
    let mut edges = Vec::new();
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            if !input.directed && j <= i {
                continue;
            }
            let forward = input
                .weights
                .get(i)
                .and_then(|row| row.get(j))
                .and_then(|value| value.as_ref());
            let reverse = input
                .weights
                .get(j)
                .and_then(|row| row.get(i))
                .and_then(|value| value.as_ref());
            let raw = if input.directed {
                forward
            } else {
                forward.or(reverse)
            };
            let Some(raw) = raw else { continue };
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                continue;
            }
            let w = Rat::parse(trimmed).map_err(|error| {
                format!(
                    "Edge {} to {}: {}",
                    label(&input.labels, i),
                    label(&input.labels, j),
                    error
                )
            })?;
            if w.is_negative() {
                return Err("Graph weights must be non-negative.".into());
            }
            edges.push(Edge { u: i, v: j, w });
        }
    }
    Ok(edges)
}

fn build_graph(n: usize, edges: &[Edge], directed: bool) -> (DiGraph<(), Rat>, Vec<NodeIndex>) {
    let mut graph = DiGraph::new();
    let nodes: Vec<NodeIndex> = (0..n).map(|_| graph.add_node(())).collect();
    for edge in edges {
        graph.add_edge(nodes[edge.u], nodes[edge.v], edge.w);
        if !directed {
            graph.add_edge(nodes[edge.v], nodes[edge.u], edge.w);
        }
    }
    (graph, nodes)
}

pub fn solve_json(source: &str) -> String {
    let input: GraphInput = match serde_json::from_str(source) {
        Ok(value) => value,
        Err(error) => {
            return serde_json::json!({"ok": false, "error": format!("Invalid request: {error}")})
                .to_string()
        }
    };

    match run(&input) {
        Ok(output) => serde_json::to_string(&output)
            .unwrap_or_else(|_| "{\"ok\":false,\"error\":\"Could not encode the result.\"}".into()),
        Err(error) => serde_json::json!({ "ok": false, "error": error }).to_string(),
    }
}

fn run(input: &GraphInput) -> Result<Output, String> {
    let n = input.labels.len();
    if n < 2 {
        return Err("Add at least two nodes.".into());
    }
    if n > 9 {
        return Err("Use at most nine nodes.".into());
    }
    let edges = parse_edges(input)?;
    if edges.is_empty() {
        return Err("Add at least one edge.".into());
    }

    match input.algorithm.as_str() {
        "dijkstra" => dijkstra(input, &edges),
        "floyd" => floyd(input, &edges),
        "kruskal" => kruskal(input, &edges),
        "prim" => prim(input, &edges),
        other => Err(format!("Unknown algorithm “{other}”.")),
    }
}

fn dijkstra(input: &GraphInput, edges: &[Edge]) -> Result<Output, String> {
    let n = input.labels.len();
    let source = input.source.min(n - 1);
    let target = input.target.min(n - 1);
    let (graph, nodes) = build_graph(n, edges, input.directed);

    let mut dist: Vec<Option<Rat>> = vec![None; n];
    let mut previous: Vec<Option<usize>> = vec![None; n];
    let mut visited = vec![false; n];
    dist[source] = Some(Rat::ZERO);

    let mut heap = BinaryHeap::new();
    heap.push(HeapEntry {
        cost: Rat::ZERO,
        node: source,
    });
    let mut steps = Vec::new();

    while let Some(HeapEntry { cost, node }) = heap.pop() {
        if visited[node] {
            continue;
        }
        if let Some(current) = dist[node] {
            if current.cmp_rat(cost) != Ordering::Equal {
                continue;
            }
        }
        visited[node] = true;
        steps.push(Step {
            text: format!(
                "Visit {} with tentative distance {}.",
                math(&label(&input.labels, node)),
                math(&weight(cost))
            ),
            latex: None,
        });

        for edge in graph.edges_directed(nodes[node], Direction::Outgoing) {
            let next = edge.target().index();
            if visited[next] {
                continue;
            }
            let edge_weight = *edge.weight();
            let candidate = cost.add(edge_weight)?;
            let improves = match dist[next] {
                None => true,
                Some(current) => rat_lt(candidate, current),
            };
            if improves {
                dist[next] = Some(candidate);
                previous[next] = Some(node);
                heap.push(HeapEntry {
                    cost: candidate,
                    node: next,
                });
                steps.push(Step {
                    text: format!(
                        "Relax {}: {} + {} = {}, so {}.",
                        math(&format!(
                            "{} \\to {}",
                            label(&input.labels, node),
                            label(&input.labels, next)
                        )),
                        math(&weight(cost)),
                        math(&weight(edge_weight)),
                        math(&weight(candidate)),
                        math(&format!(
                            "d({}) = {}",
                            label(&input.labels, next),
                            weight(candidate)
                        ))
                    ),
                    latex: None,
                });
            }
        }
    }

    let reachable = dist[target].is_some();
    let mut path = Vec::new();
    if reachable {
        let mut cursor = target;
        path.push(cursor);
        while cursor != source {
            match previous[cursor] {
                Some(prev) => {
                    cursor = prev;
                    path.push(cursor);
                }
                None => {
                    path.clear();
                    break;
                }
            }
        }
        path.reverse();
    }

    let summary = if reachable {
        let path_text = path
            .iter()
            .map(|index| label(&input.labels, *index))
            .collect::<Vec<_>>()
            .join(" \\to ");
        let distance = dist[target].unwrap();
        vec![
            format!(
                "Shortest distance {}: {}.",
                math(&format!(
                    "{} \\to {}",
                    label(&input.labels, source),
                    label(&input.labels, target)
                )),
                math(&weight(distance))
            ),
            format!("Path: {}.", math(&path_text)),
        ]
    } else {
        vec![format!(
            "There is no path from {} to {}.",
            math(&label(&input.labels, source)),
            math(&label(&input.labels, target))
        )]
    };

    let title = "Dijkstra shortest path".to_string();
    let latex = document(&title, input, edges, &summary, &steps);
    Ok(Output {
        ok: true,
        title,
        summary,
        steps,
        latex,
    })
}

fn floyd(input: &GraphInput, edges: &[Edge]) -> Result<Output, String> {
    let n = input.labels.len();
    let mut dist: Vec<Vec<Option<Rat>>> = vec![vec![None; n]; n];
    for (i, row) in dist.iter_mut().enumerate() {
        row[i] = Some(Rat::ZERO);
    }
    for edge in edges {
        let improves = match dist[edge.u][edge.v] {
            None => true,
            Some(current) => rat_lt(edge.w, current),
        };
        if improves {
            dist[edge.u][edge.v] = Some(edge.w);
        }
        if !input.directed {
            let improves = match dist[edge.v][edge.u] {
                None => true,
                Some(current) => rat_lt(edge.w, current),
            };
            if improves {
                dist[edge.v][edge.u] = Some(edge.w);
            }
        }
    }

    let mut steps = vec![Step {
        text:
            "Initialise the distance matrix; missing edges are infinite and the diagonal is zero."
                .to_string(),
        latex: Some(matrix(&input.labels, &dist)),
    }];

    for k in 0..n {
        let row_k = dist[k].clone();
        let column_k: Vec<Option<Rat>> = (0..n).map(|i| dist[i][k]).collect();
        for i in 0..n {
            for j in 0..n {
                let (Some(left), Some(right)) = (row_k[j], column_k[i]) else {
                    continue;
                };
                let candidate = left.add(right)?;
                let improves = match dist[i][j] {
                    None => true,
                    Some(current) => rat_lt(candidate, current),
                };
                if improves {
                    dist[i][j] = Some(candidate);
                }
            }
        }
        steps.push(Step {
            text: format!(
                "Allow paths through {}; update every entry with {}.",
                math(&label(&input.labels, k)),
                math(&format!(
                    "d(i,j) = \\min(d(i,j), d(i,{}) + d({},j))",
                    label(&input.labels, k),
                    label(&input.labels, k)
                ))
            ),
            latex: Some(matrix(&input.labels, &dist)),
        });
    }

    let summary: Vec<String> = dist
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let values = row
                .iter()
                .enumerate()
                .map(|(j, value)| {
                    format!(
                        "{}={}",
                        label(&input.labels, j),
                        value.map(weight).unwrap_or_else(|| "\\infty".into())
                    )
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}: {}.", math(&label(&input.labels, i)), math(&values))
        })
        .collect();

    let title = "Floyd–Warshall all-pairs shortest paths".to_string();
    let latex = document(&title, input, edges, &summary, &steps);
    Ok(Output {
        ok: true,
        title,
        summary,
        steps,
        latex,
    })
}

fn kruskal(input: &GraphInput, edges: &[Edge]) -> Result<Output, String> {
    if input.directed {
        return Err("Kruskal needs an undirected graph.".into());
    }
    let n = input.labels.len();
    let mut sorted = edges.to_vec();
    sorted.sort_by(|a, b| {
        a.w.cmp_rat(b.w)
            .then_with(|| a.u.cmp(&b.u))
            .then_with(|| a.v.cmp(&b.v))
    });

    let mut sets = UnionFind::new(n);
    let mut chosen = Vec::new();
    let mut total = Rat::ZERO;
    let mut steps = vec![Step {
        text: format!(
            "Sort the edges by weight: {}.",
            sorted
                .iter()
                .map(|edge| format!(
                    "{}{} ({})",
                    label(&input.labels, edge.u),
                    label(&input.labels, edge.v),
                    weight(edge.w)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        latex: None,
    }];

    for edge in &sorted {
        let joined = sets.union(edge.u, edge.v);
        let pair = format!(
            "{}{}",
            label(&input.labels, edge.u),
            label(&input.labels, edge.v)
        );
        if joined {
            chosen.push(*edge);
            total = total.add(edge.w)?;
            steps.push(Step {
                text: format!(
                    "Accept {} with weight {}; running total {}.",
                    math(&pair),
                    math(&weight(edge.w)),
                    math(&weight(total))
                ),
                latex: None,
            });
        } else {
            steps.push(Step {
                text: format!(
                    "Reject {} with weight {}; it would form a cycle.",
                    math(&pair),
                    math(&weight(edge.w))
                ),
                latex: None,
            });
        }
    }

    let connected = chosen.len() == n - 1;
    let summary = vec![
        if connected {
            format!("Minimum spanning tree total: {}.", math(&weight(total)))
        } else {
            "The graph is disconnected; a spanning tree is not possible.".to_string()
        },
        format!(
            "Edges: {}.",
            if chosen.is_empty() {
                "none".to_string()
            } else {
                chosen
                    .iter()
                    .map(|edge| {
                        format!(
                            "{}{} ({})",
                            label(&input.labels, edge.u),
                            label(&input.labels, edge.v),
                            weight(edge.w)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ),
    ];

    let title = "Kruskal minimum spanning tree".to_string();
    let latex = document(&title, input, edges, &summary, &steps);
    Ok(Output {
        ok: true,
        title,
        summary,
        steps,
        latex,
    })
}

fn prim(input: &GraphInput, edges: &[Edge]) -> Result<Output, String> {
    if input.directed {
        return Err("Prim needs an undirected graph.".into());
    }
    let n = input.labels.len();
    let start = input.source.min(n - 1);
    let mut visited = vec![false; n];
    visited[start] = true;
    let mut chosen = Vec::new();
    let mut total = Rat::ZERO;
    let mut steps = vec![Step {
        text: format!("Start at {}.", math(&label(&input.labels, start))),
        latex: None,
    }];

    while chosen.len() < n - 1 {
        let mut best: Option<Edge> = None;
        for edge in edges {
            let crossing =
                (visited[edge.u] && !visited[edge.v]) || (visited[edge.v] && !visited[edge.u]);
            if !crossing {
                continue;
            }
            if best.is_none_or(|current| rat_lt(edge.w, current.w)) {
                best = Some(*edge);
            }
        }
        let Some(edge) = best else { break };
        let next = if visited[edge.u] { edge.v } else { edge.u };
        visited[next] = true;
        chosen.push(edge);
        total = total.add(edge.w)?;
        steps.push(Step {
            text: format!(
                "Add {} with weight {}; running total {}.",
                math(&format!(
                    "{}{}",
                    label(&input.labels, edge.u),
                    label(&input.labels, edge.v)
                )),
                math(&weight(edge.w)),
                math(&weight(total))
            ),
            latex: None,
        });
    }

    let connected = chosen.len() == n - 1;
    let summary = vec![
        if connected {
            format!("Minimum spanning tree total: {}.", math(&weight(total)))
        } else {
            format!(
                "The graph is disconnected from {}.",
                math(&label(&input.labels, start))
            )
        },
        format!(
            "Edges: {}.",
            if chosen.is_empty() {
                "none".to_string()
            } else {
                chosen
                    .iter()
                    .map(|edge| {
                        format!(
                            "{}{} ({})",
                            label(&input.labels, edge.u),
                            label(&input.labels, edge.v),
                            weight(edge.w)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ),
    ];

    let title = "Prim minimum spanning tree".to_string();
    let latex = document(&title, input, edges, &summary, &steps);
    Ok(Output {
        ok: true,
        title,
        summary,
        steps,
        latex,
    })
}

fn matrix(labels: &[String], dist: &[Vec<Option<Rat>>]) -> String {
    let header = std::iter::once(String::new())
        .chain(labels.iter().map(|name| text(name)))
        .collect::<Vec<_>>()
        .join(" & ");
    let body = dist
        .iter()
        .enumerate()
        .map(|(index, row)| {
            std::iter::once(text(&labels[index]))
                .chain(
                    row.iter()
                        .map(|value| value.map(weight).unwrap_or_else(|| "\\infty".to_string())),
                )
                .collect::<Vec<_>>()
                .join(" & ")
        })
        .collect::<Vec<_>>()
        .join(" \\\\\n");
    format!(
        "\\begin{{array}}{{c|{}}}\n{} \\\\ \\hline\n{}\n\\end{{array}}",
        "c".repeat(labels.len()),
        header,
        body
    )
}

fn document(
    title: &str,
    input: &GraphInput,
    edges: &[Edge],
    summary: &[String],
    steps: &[Step],
) -> String {
    let edge_list = edges
        .iter()
        .map(|edge| {
            format!(
                "{}{}{}\\;(\\text{{weight }} {})",
                text(&label(&input.labels, edge.u)),
                if input.directed { "\\to" } else { "--" },
                text(&label(&input.labels, edge.v)),
                weight(edge.w)
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let summary_body = summary
        .iter()
        .map(|line| format!("\\item {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let steps_body = steps
        .iter()
        .map(|step| {
            let extra = step
                .latex
                .as_ref()
                .map(|latex| format!("\n\\[\n{latex}\n\\]"))
                .unwrap_or_default();
            format!("\\item {}{}", step.text, extra)
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "\\documentclass[11pt]{{article}}
\\usepackage[a4paper,margin=2.2cm]{{geometry}}
\\usepackage{{amsmath,amssymb}}
\\setlength{{\\parindent}}{{0pt}}
\\begin{{document}}
\\section*{{{title}}}
Nodes: {}. Edges: {}.

\\subsection*{{Result}}
\\begin{{itemize}}
{summary_body}
\\end{{itemize}}

\\subsection*{{Working}}
\\begin{{itemize}}
{steps_body}
\\end{{itemize}}
\\end{{document}}
",
        input.labels.join(", "),
        if edge_list.is_empty() {
            "none".to_string()
        } else {
            edge_list
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_shortest_path() {
        let input: GraphInput = serde_json::from_str(
            r#"{
                "labels": ["A","B","C"],
                "directed": false,
                "weights": [["", "2", "5"], ["2", "", "1"], ["5", "1", ""]],
                "algorithm": "dijkstra",
                "source": 0,
                "target": 2
            }"#,
        )
        .unwrap();
        let output = run(&input).unwrap();
        assert!(output.summary[0].contains("3"));
    }

    #[test]
    fn reports_overflow_instead_of_panicking() {
        let input: GraphInput = serde_json::from_str(
            r#"{
                "labels": ["A","B","C"],
                "directed": false,
                "weights": [
                    ["", "9223372036854775806", ""],
                    ["9223372036854775806", "", "9223372036854775806"],
                    ["", "9223372036854775806", ""]
                ],
                "algorithm": "dijkstra",
                "source": 0,
                "target": 2
            }"#,
        )
        .unwrap();
        let Err(error) = run(&input) else {
            panic!("expected an overflow error");
        };
        assert!(error.contains("too large"));
    }

    #[test]
    fn builds_spanning_tree() {
        let input: GraphInput = serde_json::from_str(
            r#"{
                "labels": ["A","B","C"],
                "directed": false,
                "weights": [["", "2", "5"], ["2", "", "1"], ["5", "1", ""]],
                "algorithm": "kruskal",
                "source": 0,
                "target": 2
            }"#,
        )
        .unwrap();
        let output = run(&input).unwrap();
        assert!(output.summary[0].contains("3"));
    }
}
