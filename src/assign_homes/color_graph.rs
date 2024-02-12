use std::collections::{HashMap, HashSet};

use keyed_priority_queue::KeyedPriorityQueue;

use crate::assign_homes::build_interference::{self as prev};
use crate::assign_homes::build_move::MoveGraph;
use crate::var::Var;

use petgraph::graph::NodeIndex;

type Color = u32;

#[derive(Clone, Eq, PartialEq, Debug)]
struct Priority {
    saturation: usize,
    move_saturation: usize,
}

impl Priority {
    fn new() -> Self {
        Priority {
            saturation: 0,
            move_saturation: 0,
        }
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.saturation
            .cmp(&other.saturation)
            .then(self.move_saturation.cmp(&other.move_saturation))
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.saturation
                .partial_cmp(&other.saturation)?
                .then(self.move_saturation.partial_cmp(&other.move_saturation)?),
        )
    }
}

pub fn color_graph(interference_graph: prev::Graph, move_graph: &MoveGraph) -> HashMap<Var, Color> {
    let mut pqueue = KeyedPriorityQueue::new();

    for index in interference_graph.node_indices() {
        pqueue.push(index, Priority::new());
    }

    let mut color_map = HashMap::new();

    while let Some((index, _)) = pqueue.pop() {
        if !color_map.contains_key(&interference_graph[index]) {
            let color = find_least_color(&interference_graph, move_graph, &color_map, index);
            color_map.insert(interference_graph[index].clone(), color);
            update_saturation(&mut pqueue, &interference_graph, &color_map, index);
            update_move_saturation(
                &mut pqueue,
                &interference_graph,
                &move_graph,
                &color_map,
                index,
            );
        }
    }

    color_map
}

fn find_least_color(
    graph: &prev::Graph,
    move_graph: &MoveGraph,
    color_map: &HashMap<Var, Color>,
    index: NodeIndex,
) -> Color {
    let mut color = 0;
    let conflicts = find_conflicting_colors(graph, color_map, index);

    for relative in find_move_related_colors(move_graph, color_map, &graph[index]) {
        if !conflicts.contains(&relative) {
            return relative;
        }
    }

    while conflicts.contains(&color) {
        color += 1;
    }

    color
}

fn find_move_related_colors(
    move_graph: &MoveGraph,
    color_map: &HashMap<Var, Color>,
    var: &Var,
) -> HashSet<Color> {
    let mut relatives = HashSet::new();
    for relative in color_map
        .keys()
        .filter(|relative| move_graph.move_related(var, relative))
    {
        relatives.insert(color_map[relative]);
    }
    relatives
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
    pqueue: &mut KeyedPriorityQueue<NodeIndex<u32>, Priority>,
    graph: &prev::Graph,
    color_map: &HashMap<Var, Color>,
    index: NodeIndex,
) {
    for adj in graph
        .neighbors(index)
        .filter(|idx| !color_map.contains_key(&graph[*idx]))
    {
        let move_saturation = pqueue
            .get_priority(&adj)
            .expect("Uncolored neighbor not in pqueue?")
            .move_saturation;
        pqueue
            .set_priority(
                &adj,
                Priority {
                    saturation: saturation(graph, color_map, adj),
                    move_saturation,
                },
            )
            .unwrap();
    }
}

fn update_move_saturation(
    pqueue: &mut KeyedPriorityQueue<NodeIndex<u32>, Priority>,
    graph: &prev::Graph,
    move_graph: &MoveGraph,
    color_map: &HashMap<Var, Color>,
    index: NodeIndex,
) {
    let a = &graph[index];
    for b in graph.node_indices().filter(|idx| {
        let v = &graph[*idx];
        !color_map.contains_key(v)
            && move_graph.move_related(&a, v)
            && !graph.contains_edge(index, *idx)
    }) {
        let mut priority = pqueue
            .get_priority(&b)
            .expect("Uncolored node not in pqueue?")
            .clone();
        priority.move_saturation += 1;
        pqueue.set_priority(&b, priority).unwrap();

        dbg!(&pqueue);
    }
}

fn saturation(graph: &prev::Graph, color_map: &HashMap<Var, Color>, index: NodeIndex) -> usize {
    find_conflicting_colors(graph, color_map, index).len()
}
