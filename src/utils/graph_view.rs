use petgraph::{Direction, graph::DiGraph};

use crate::references::Instance;

pub struct Vf2GraphView<'a>(pub &'a DiGraph<Instance, crate::references::Ref>);

impl<'a> vf2::Graph for Vf2GraphView<'a> {
    type NodeLabel = Instance;
    type EdgeLabel = crate::references::Ref;

    fn is_directed(&self) -> bool {
        true
    }

    fn node_count(&self) -> usize {
        self.0.node_count()
    }

    fn node_label(&self, node: vf2::NodeIndex) -> Option<&Self::NodeLabel> {
        self.0.node_weight(petgraph::graph::NodeIndex::new(node))
    }

    fn neighbors(
        &self,
        node: vf2::NodeIndex,
        direction: vf2::Direction,
    ) -> impl Iterator<Item = vf2::NodeIndex> {
        self.0
            .neighbors_directed(
                petgraph::graph::NodeIndex::new(node),
                match direction {
                    vf2::Direction::Outgoing => Direction::Outgoing,
                    vf2::Direction::Incoming => Direction::Incoming,
                },
            )
            .map(|neighbor| neighbor.index())
    }

    fn contains_edge(&self, source: vf2::NodeIndex, target: vf2::NodeIndex) -> bool {
        self.0.contains_edge(
            petgraph::graph::NodeIndex::new(source),
            petgraph::graph::NodeIndex::new(target),
        )
    }

    fn edge_label(
        &self,
        source: vf2::NodeIndex,
        target: vf2::NodeIndex,
    ) -> Option<&Self::EdgeLabel> {
        self.0
            .find_edge(
                petgraph::graph::NodeIndex::new(source),
                petgraph::graph::NodeIndex::new(target),
            )
            .and_then(|edge| self.0.edge_weight(edge))
    }
}
