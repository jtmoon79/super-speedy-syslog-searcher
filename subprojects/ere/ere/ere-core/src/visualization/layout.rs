//! Implements algorithms for the spatial layout of the visualized graphs

use crate::visualization::LatexGraph;

pub trait BuildLayout {
    /// Returns a list of (x, y) coordinates for each node
    ///
    /// Returned list should be in the same order as the nodes in the graph
    fn layout(&self) -> Vec<(f64, f64)>;
}

/// For DAGs, EXCEPT that self-loops are allowed
///
/// https://en.wikipedia.org/wiki/Layered_graph_drawing
pub struct DAGLayout<'a> {
    depths: Vec<usize>,
    max_depth: usize,
    graph: &'a LatexGraph,
}
impl<'a> DAGLayout<'a> {
    fn check_dag(graph: &'a LatexGraph) -> bool {
        if graph.states.len() == 0 {
            return false;
        }
        let mut done = vec![false; graph.states.len()];
        let mut active = vec![false; graph.states.len()];
        enum StackItem {
            /// (index, depth)
            PreVisit(usize, usize),
            PostVisit(usize),
        }
        let mut stack = vec![StackItem::PreVisit(0, 0)];

        while let Some(item) = stack.pop() {
            match item {
                StackItem::PreVisit(node, depth) => {
                    if done[node] {
                        continue;
                    } else if active[node] {
                        return false;
                    }
                    active[node] = true;
                    stack.push(StackItem::PostVisit(node));
                    for tr in &graph.states[node].transitions {
                        if tr.to == node {
                            // while this self-loop technically makes it not a DAG
                            // that doesn't matter for the layout so we allow it
                            continue;
                        }
                        stack.push(StackItem::PreVisit(tr.to, depth + 1));
                    }
                }
                StackItem::PostVisit(node) => {
                    done[node] = true;
                }
            }
        }

        return done.iter().all(|b| *b);
    }
    /// Returns `None` if the graph is not DAG
    pub fn new(graph: &'a LatexGraph) -> Option<Self> {
        if !Self::check_dag(graph) {
            return None;
        }
        let mut max_depth = 0;
        let mut depths = vec![usize::MAX; graph.states.len()];
        let mut stack = vec![(0, 0)];
        while let Some((node, depth)) = stack.pop() {
            if depth > depths[node] || depths[node] == usize::MAX {
                depths[node] = depth;
                max_depth = std::cmp::max(max_depth, depth);
                stack.extend(
                    graph.states[node]
                        .transitions
                        .iter()
                        .filter(|tr| tr.to != node)
                        .map(|tr| (tr.to, depth + 1)),
                );
            }
        }

        return Some(DAGLayout {
            max_depth,
            depths,
            graph,
        });
    }
}
impl<'a> BuildLayout for DAGLayout<'a> {
    fn layout(&self) -> Vec<(f64, f64)> {
        let unique_depths = self.max_depth + 1;

        enum LayerNode {
            Real {
                node: usize,
                transitions: Vec<usize>,
            },
        }
        let mut depth_sizes = vec![0; unique_depths + 1]; // # of nodes at each depth
        for depth in self.depths.iter() {
            depth_sizes[*depth] += 1;
        }

        let mut depth_used = vec![0; unique_depths + 1]; // # of used nodes at each depth
        self.depths
            .iter()
            .map(|depth| {
                let x = *depth as f64;
                let y = -(depth_sizes[*depth] as f64) / 2.0 + depth_used[*depth] as f64;

                depth_used[*depth] += 1;
                (x, y)
            })
            .collect()
    }
}
