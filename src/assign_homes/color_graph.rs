use std::collections::{BTreeSet, HashMap, HashSet};

use crate::assign_homes::build_interference::{self as prev};
use crate::var::Var;

use petgraph::graph::NodeIndex;

type Color = u32;

#[derive(Clone, Eq, PartialEq)]
struct Node {
    index: NodeIndex<u32>,
    saturation: usize,
}

impl Node {
    fn new(index: NodeIndex) -> Self {
        Node {
            index,
            saturation: 0,
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.saturation.cmp(&other.saturation).then(self.index.cmp(&other.index))
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.saturation.partial_cmp(&other.saturation)?.then(self.index.partial_cmp(&other.index)?))
    }
}

pub fn color_graph(interference_graph: prev::Graph) -> HashMap<Var, Color> {
    let mut pqueue = BTreeSet::new();

    for index in interference_graph.node_indices() {
        pqueue.insert(Node::new(index));
    }

    let mut color_map = HashMap::new();

    while let Some(node) = pqueue.pop_last() {
        if !color_map.contains_key(&interference_graph[node.index]) {
            let color = find_least_color(&interference_graph, &color_map, node.index);
            color_map.insert(interference_graph[node.index].clone(), color);
            update_saturation(&mut pqueue, &interference_graph, &color_map, node.index);
        }
    }

    color_map
}

fn find_least_color(
    graph: &prev::Graph,
    color_map: &HashMap<Var, Color>,
    index: NodeIndex,
) -> Color {
    let mut color = 0;
    let conflicts = find_conflicting_colors(graph, color_map, index);

    while conflicts.contains(&color) {
        color += 1;
    }

    color
}

fn find_conflicting_colors(
    graph: &prev::Graph,
    color_map: &HashMap<Var, Color>,
    index: NodeIndex,
) -> HashSet<Color> {
    let mut conflicts = HashSet::new();
    for adj in graph.neighbors(index) {
        if let Some(color) = color_map.get(&graph[adj]) {
            conflicts.insert(*color);
        }
    }
    conflicts
}

fn update_saturation(
    pqueue: &mut BTreeSet<Node>,
    graph: &prev::Graph,
    color_map: &HashMap<Var, Color>,
    index: NodeIndex,
) {
    for adj in graph
        .neighbors(index)
        .filter(|idx| !color_map.contains_key(&graph[*idx]))
    {
        pqueue.insert(Node {
            index: adj,
            saturation: saturation(graph, color_map, index),
        });
    }
}

fn saturation(graph: &prev::Graph, color_map: &HashMap<Var, Color>, index: NodeIndex) -> usize {
    find_conflicting_colors(graph, color_map, index).len()
}
