use bevy_ecs::{prelude::World, resource::Resource};
use bevy_platform::collections::HashMap;
use core::fmt::Debug;

use crate::{
    frame_graph::FrameGraph,
    render_graph::{
        GraphInput, InternedRenderSubGraph, IntoRenderNodeArray, RenderLabel, RenderSubGraph,
    },
};

use super::{
    Edge, EdgeExistence, InternedRenderLabel, NodeState, Setup, SetupGraphContext, SetupGraphError,
    SetupRunError, SlotInfo, SlotLabel,
};

/// The render graph configures the modular and re-usable render logic.
///
/// It is a retained and stateless (nodes themselves may have their own internal state) structure,
/// which can not be modified while it is executed by the graph runner.
///
/// The render graph runner is responsible for executing the entire graph each frame.
/// It will execute each node in the graph in the correct order, based on the edges between the nodes.
///
/// It consists of three main components: [`Nodes`](Setup), [`Edges`](Edge)
/// and [`Slots`](super::SlotType).
///
/// Nodes are responsible for generating draw calls and operating on input and output slots.
/// Edges specify the order of execution for nodes and connect input and output slots together.
/// Slots describe the render resources created or used by the nodes.
///
/// Additionally a render graph can contain multiple sub graphs, which are run by the
/// corresponding nodes. Every render graph can have its own optional input node.
///
/// ## Example
/// Here is a simple render graph example with two nodes connected by a node edge.
/// ```ignore
/// # TODO: Remove when #10645 is fixed
/// # use bevy_app::prelude::*;
/// # use bevy_ecs::prelude::World;
/// # use bevy_render::render_graph::{SetupGraph, RenderLabel, Setup, SetupGraphContext, NodeRunError};
/// # use bevy_render::renderer::RenderContext;
/// #
/// #[derive(RenderLabel)]
/// enum Labels {
///     A,
///     B,
/// }
///
/// # struct MyNode;
/// #
/// # impl Setup for MyNode {
/// #     fn run(&self, graph: &mut SetupGraphContext, render_context: &mut RenderContext, world: &World) -> Result<(), NodeRunError> {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// let mut graph = SetupGraph::default();
/// graph.add_node(Labels::A, MyNode);
/// graph.add_node(Labels::B, MyNode);
/// graph.add_node_edge(Labels::B, Labels::A);
/// ```
#[derive(Resource, Default)]
pub struct SetupGraph {
    nodes: HashMap<InternedRenderLabel, NodeState>,
    sub_graphs: HashMap<InternedRenderSubGraph, SetupGraph>,
}

impl SetupGraph {
    /// Updates all nodes and sub graphs of the render graph. Should be called before executing it.
    pub fn update(&mut self, world: &mut World) {
        for node in self.nodes.values_mut() {
            node.node.update(world);
        }

        for sub_graph in self.sub_graphs.values_mut() {
            sub_graph.update(world);
        }
    }

    /// Creates an [`GraphInputNode`] with the specified slots if not already present.
    pub fn set_input(&mut self, inputs: Vec<SlotInfo>) {
        assert!(
            matches!(
                self.get_node_state(GraphInput),
                Err(SetupGraphError::InvalidNode(_))
            ),
            "Graph already has an input node"
        );

        self.add_node(GraphInput, GraphInputSetup { inputs });
    }

    /// Returns the [`NodeState`] of the input node of this graph.
    ///
    /// # See also
    ///
    /// - [`input_node`](Self::input_node) for an unchecked version.
    #[inline]
    pub fn get_input_node(&self) -> Option<&NodeState> {
        self.get_node_state(GraphInput).ok()
    }

    /// Returns the [`NodeState`] of the input node of this graph.
    ///
    /// # Panics
    ///
    /// Panics if there is no input node set.
    ///
    /// # See also
    ///
    /// - [`get_input_node`](Self::get_input_node) for a version which returns an [`Option`] instead.
    #[inline]
    pub fn input_node(&self) -> &NodeState {
        self.get_input_node().unwrap()
    }

    /// Adds the `node` with the `label` to the graph.
    /// If the label is already present replaces it instead.
    pub fn add_node<T>(&mut self, label: impl RenderLabel, node: T)
    where
        T: Setup,
    {
        let label = label.intern();
        let node_state = NodeState::new(label, node);
        self.nodes.insert(label, node_state);
    }

    /// Add `node_edge`s based on the order of the given `edges` array.
    ///
    /// Defining an edge that already exists is not considered an error with this api.
    /// It simply won't create a new edge.
    pub fn add_node_edges<const N: usize>(&mut self, edges: impl IntoRenderNodeArray<N>) {
        for window in edges.into_array().windows(2) {
            let [a, b] = window else {
                break;
            };
            if let Err(err) = self.try_add_node_edge(*a, *b) {
                match err {
                    // Already existing edges are very easy to produce with this api
                    // and shouldn't cause a panic
                    SetupGraphError::EdgeAlreadyExists(_) => {}
                    _ => panic!("{err:?}"),
                }
            }
        }
    }

    /// Removes the `node` with the `label` from the graph.
    /// If the label does not exist, nothing happens.
    pub fn remove_node(&mut self, label: impl RenderLabel) -> Result<(), SetupGraphError> {
        let label = label.intern();
        if let Some(node_state) = self.nodes.remove(&label) {
            // Remove all edges from other nodes to this one. Note that as we're removing this
            // node, we don't need to remove its input edges
            for input_edge in node_state.edges.input_edges() {
                match input_edge {
                    Edge::SlotEdge { output_node, .. }
                    | Edge::NodeEdge {
                        input_node: _,
                        output_node,
                    } => {
                        if let Ok(output_node) = self.get_node_state_mut(*output_node) {
                            output_node.edges.remove_output_edge(input_edge.clone())?;
                        }
                    }
                }
            }
            // Remove all edges from this node to other nodes. Note that as we're removing this
            // node, we don't need to remove its output edges
            for output_edge in node_state.edges.output_edges() {
                match output_edge {
                    Edge::SlotEdge {
                        output_node: _,
                        output_index: _,
                        input_node,
                        input_index: _,
                    }
                    | Edge::NodeEdge {
                        output_node: _,
                        input_node,
                    } => {
                        if let Ok(input_node) = self.get_node_state_mut(*input_node) {
                            input_node.edges.remove_input_edge(output_edge.clone())?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Retrieves the [`NodeState`] referenced by the `label`.
    pub fn get_node_state(&self, label: impl RenderLabel) -> Result<&NodeState, SetupGraphError> {
        let label = label.intern();
        self.nodes
            .get(&label)
            .ok_or(SetupGraphError::InvalidNode(label))
    }

    /// Retrieves the [`NodeState`] referenced by the `label` mutably.
    pub fn get_node_state_mut(
        &mut self,
        label: impl RenderLabel,
    ) -> Result<&mut NodeState, SetupGraphError> {
        let label = label.intern();
        self.nodes
            .get_mut(&label)
            .ok_or(SetupGraphError::InvalidNode(label))
    }

    pub fn get_node<T>(&self, label: impl RenderLabel) -> Result<&T, SetupGraphError>
    where
        T: Setup,
    {
        self.get_node_state(label).and_then(|n| n.node())
    }

    /// Retrieves the [`Setup`] referenced by the `label` mutably.
    pub fn get_node_mut<T>(&mut self, label: impl RenderLabel) -> Result<&mut T, SetupGraphError>
    where
        T: Setup,
    {
        self.get_node_state_mut(label).and_then(|n| n.node_mut())
    }

    /// Adds the [`Edge::SlotEdge`] to the graph. This guarantees that the `output_node`
    /// is run before the `input_node` and also connects the `output_slot` to the `input_slot`.
    ///
    /// Fails if any invalid [`RenderLabel`]s or [`SlotLabel`]s are given.
    ///
    /// # See also
    ///
    /// - [`add_slot_edge`](Self::add_slot_edge) for an infallible version.
    pub fn try_add_slot_edge(
        &mut self,
        output_node: impl RenderLabel,
        output_slot: impl Into<SlotLabel>,
        input_node: impl RenderLabel,
        input_slot: impl Into<SlotLabel>,
    ) -> Result<(), SetupGraphError> {
        let output_slot = output_slot.into();
        let input_slot = input_slot.into();

        let output_node = output_node.intern();
        let input_node = input_node.intern();

        let output_index = self
            .get_node_state(output_node)?
            .output_slots
            .get_slot_index(output_slot.clone())
            .ok_or(SetupGraphError::InvalidOutputNodeSlot(output_slot))?;
        let input_index = self
            .get_node_state(input_node)?
            .input_slots
            .get_slot_index(input_slot.clone())
            .ok_or(SetupGraphError::InvalidInputNodeSlot(input_slot))?;

        let edge = Edge::SlotEdge {
            output_node,
            output_index,
            input_node,
            input_index,
        };

        self.validate_edge(&edge, EdgeExistence::DoesNotExist)?;

        {
            let output_node = self.get_node_state_mut(output_node)?;
            output_node.edges.add_output_edge(edge.clone())?;
        }
        let input_node = self.get_node_state_mut(input_node)?;
        input_node.edges.add_input_edge(edge)?;

        Ok(())
    }

    /// Adds the [`Edge::SlotEdge`] to the graph. This guarantees that the `output_node`
    /// is run before the `input_node` and also connects the `output_slot` to the `input_slot`.
    ///
    /// # Panics
    ///
    /// Any invalid [`RenderLabel`]s or [`SlotLabel`]s are given.
    ///
    /// # See also
    ///
    /// - [`try_add_slot_edge`](Self::try_add_slot_edge) for a fallible version.
    pub fn add_slot_edge(
        &mut self,
        output_node: impl RenderLabel,
        output_slot: impl Into<SlotLabel>,
        input_node: impl RenderLabel,
        input_slot: impl Into<SlotLabel>,
    ) {
        self.try_add_slot_edge(output_node, output_slot, input_node, input_slot)
            .unwrap();
    }

    /// Removes the [`Edge::SlotEdge`] from the graph. If any nodes or slots do not exist then
    /// nothing happens.
    pub fn remove_slot_edge(
        &mut self,
        output_node: impl RenderLabel,
        output_slot: impl Into<SlotLabel>,
        input_node: impl RenderLabel,
        input_slot: impl Into<SlotLabel>,
    ) -> Result<(), SetupGraphError> {
        let output_slot = output_slot.into();
        let input_slot = input_slot.into();

        let output_node = output_node.intern();
        let input_node = input_node.intern();

        let output_index = self
            .get_node_state(output_node)?
            .output_slots
            .get_slot_index(output_slot.clone())
            .ok_or(SetupGraphError::InvalidOutputNodeSlot(output_slot))?;
        let input_index = self
            .get_node_state(input_node)?
            .input_slots
            .get_slot_index(input_slot.clone())
            .ok_or(SetupGraphError::InvalidInputNodeSlot(input_slot))?;

        let edge = Edge::SlotEdge {
            output_node,
            output_index,
            input_node,
            input_index,
        };

        self.validate_edge(&edge, EdgeExistence::Exists)?;

        {
            let output_node = self.get_node_state_mut(output_node)?;
            output_node.edges.remove_output_edge(edge.clone())?;
        }
        let input_node = self.get_node_state_mut(input_node)?;
        input_node.edges.remove_input_edge(edge)?;

        Ok(())
    }

    /// Adds the [`Edge::NodeEdge`] to the graph. This guarantees that the `output_node`
    /// is run before the `input_node`.
    ///
    /// Fails if any invalid [`RenderLabel`] is given.
    ///
    /// # See also
    ///
    /// - [`add_node_edge`](Self::add_node_edge) for an infallible version.
    pub fn try_add_node_edge(
        &mut self,
        output_node: impl RenderLabel,
        input_node: impl RenderLabel,
    ) -> Result<(), SetupGraphError> {
        let output_node = output_node.intern();
        let input_node = input_node.intern();

        let edge = Edge::NodeEdge {
            output_node,
            input_node,
        };

        self.validate_edge(&edge, EdgeExistence::DoesNotExist)?;

        {
            let output_node = self.get_node_state_mut(output_node)?;
            output_node.edges.add_output_edge(edge.clone())?;
        }
        let input_node = self.get_node_state_mut(input_node)?;
        input_node.edges.add_input_edge(edge)?;

        Ok(())
    }

    /// Adds the [`Edge::NodeEdge`] to the graph. This guarantees that the `output_node`
    /// is run before the `input_node`.
    ///
    /// # Panics
    ///
    /// Panics if any invalid [`RenderLabel`] is given.
    ///
    /// # See also
    ///
    /// - [`try_add_node_edge`](Self::try_add_node_edge) for a fallible version.
    pub fn add_node_edge(&mut self, output_node: impl RenderLabel, input_node: impl RenderLabel) {
        self.try_add_node_edge(output_node, input_node).unwrap();
    }

    /// Removes the [`Edge::NodeEdge`] from the graph. If either node does not exist then nothing
    /// happens.
    pub fn remove_node_edge(
        &mut self,
        output_node: impl RenderLabel,
        input_node: impl RenderLabel,
    ) -> Result<(), SetupGraphError> {
        let output_node = output_node.intern();
        let input_node = input_node.intern();

        let edge = Edge::NodeEdge {
            output_node,
            input_node,
        };

        self.validate_edge(&edge, EdgeExistence::Exists)?;

        {
            let output_node = self.get_node_state_mut(output_node)?;
            output_node.edges.remove_output_edge(edge.clone())?;
        }
        let input_node = self.get_node_state_mut(input_node)?;
        input_node.edges.remove_input_edge(edge)?;

        Ok(())
    }

    /// Verifies that the edge existence is as expected and
    /// checks that slot edges are connected correctly.
    pub fn validate_edge(
        &mut self,
        edge: &Edge,
        should_exist: EdgeExistence,
    ) -> Result<(), SetupGraphError> {
        if should_exist == EdgeExistence::Exists && !self.has_edge(edge) {
            return Err(SetupGraphError::EdgeDoesNotExist(edge.clone()));
        } else if should_exist == EdgeExistence::DoesNotExist && self.has_edge(edge) {
            return Err(SetupGraphError::EdgeAlreadyExists(edge.clone()));
        }

        match *edge {
            Edge::SlotEdge {
                output_node,
                output_index,
                input_node,
                input_index,
            } => {
                let output_node_state = self.get_node_state(output_node)?;
                let input_node_state = self.get_node_state(input_node)?;

                let output_slot = output_node_state
                    .output_slots
                    .get_slot(output_index)
                    .ok_or(SetupGraphError::InvalidOutputNodeSlot(SlotLabel::Index(
                        output_index,
                    )))?;
                let input_slot = input_node_state.input_slots.get_slot(input_index).ok_or(
                    SetupGraphError::InvalidInputNodeSlot(SlotLabel::Index(input_index)),
                )?;

                if let Some(Edge::SlotEdge {
                    output_node: current_output_node,
                    ..
                }) = input_node_state.edges.input_edges().iter().find(|e| {
                    if let Edge::SlotEdge {
                        input_index: current_input_index,
                        ..
                    } = e
                    {
                        input_index == *current_input_index
                    } else {
                        false
                    }
                }) {
                    if should_exist == EdgeExistence::DoesNotExist {
                        return Err(SetupGraphError::NodeInputSlotAlreadyOccupied {
                            node: input_node,
                            input_slot: input_index,
                            occupied_by_node: *current_output_node,
                        });
                    }
                }

                if output_slot.slot_type != input_slot.slot_type {
                    return Err(SetupGraphError::MismatchedNodeSlots {
                        output_node,
                        output_slot: output_index,
                        input_node,
                        input_slot: input_index,
                    });
                }
            }
            Edge::NodeEdge { .. } => { /* nothing to validate here */ }
        }

        Ok(())
    }

    /// Checks whether the `edge` already exists in the graph.
    pub fn has_edge(&self, edge: &Edge) -> bool {
        let output_node_state = self.get_node_state(edge.get_output_node());
        let input_node_state = self.get_node_state(edge.get_input_node());
        if let Ok(output_node_state) = output_node_state {
            if output_node_state.edges.output_edges().contains(edge) {
                if let Ok(input_node_state) = input_node_state {
                    if input_node_state.edges.input_edges().contains(edge) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Returns an iterator over the [`NodeStates`](NodeState).
    pub fn iter_nodes(&self) -> impl Iterator<Item = &NodeState> {
        self.nodes.values()
    }

    /// Returns an iterator over the [`NodeStates`](NodeState), that allows modifying each value.
    pub fn iter_nodes_mut(&mut self) -> impl Iterator<Item = &mut NodeState> {
        self.nodes.values_mut()
    }

    /// Returns an iterator over the sub graphs.
    pub fn iter_sub_graphs(&self) -> impl Iterator<Item = (InternedRenderSubGraph, &SetupGraph)> {
        self.sub_graphs.iter().map(|(name, graph)| (*name, graph))
    }

    /// Returns an iterator over the sub graphs, that allows modifying each value.
    pub fn iter_sub_graphs_mut(
        &mut self,
    ) -> impl Iterator<Item = (InternedRenderSubGraph, &mut SetupGraph)> {
        self.sub_graphs
            .iter_mut()
            .map(|(name, graph)| (*name, graph))
    }

    /// Returns an iterator over a tuple of the input edges and the corresponding output nodes
    /// for the node referenced by the label.
    pub fn iter_node_inputs(
        &self,
        label: impl RenderLabel,
    ) -> Result<impl Iterator<Item = (&Edge, &NodeState)>, SetupGraphError> {
        let node = self.get_node_state(label)?;
        Ok(node
            .edges
            .input_edges()
            .iter()
            .map(|edge| (edge, edge.get_output_node()))
            .map(move |(edge, output_node)| (edge, self.get_node_state(output_node).unwrap())))
    }

    pub fn iter_node_outputs(
        &self,
        label: impl RenderLabel,
    ) -> Result<impl Iterator<Item = (&Edge, &NodeState)>, SetupGraphError> {
        let node = self.get_node_state(label)?;
        Ok(node
            .edges
            .output_edges()
            .iter()
            .map(|edge| (edge, edge.get_input_node()))
            .map(move |(edge, input_node)| (edge, self.get_node_state(input_node).unwrap())))
    }

    /// Adds the `sub_graph` with the `label` to the graph.
    /// If the label is already present replaces it instead.
    pub fn add_sub_graph(&mut self, label: impl RenderSubGraph, sub_graph: SetupGraph) {
        self.sub_graphs.insert(label.intern(), sub_graph);
    }

    /// Removes the `sub_graph` with the `label` from the graph.
    /// If the label does not exist then nothing happens.
    pub fn remove_sub_graph(&mut self, label: impl RenderSubGraph) {
        self.sub_graphs.remove(&label.intern());
    }

    /// Retrieves the sub graph corresponding to the `label`.
    pub fn get_sub_graph(&self, label: impl RenderSubGraph) -> Option<&SetupGraph> {
        self.sub_graphs.get(&label.intern())
    }

    /// Retrieves the sub graph corresponding to the `label` mutably.
    pub fn get_sub_graph_mut(&mut self, label: impl RenderSubGraph) -> Option<&mut SetupGraph> {
        self.sub_graphs.get_mut(&label.intern())
    }

    /// Retrieves the sub graph corresponding to the `label`.
    ///
    /// # Panics
    ///
    /// Panics if any invalid subgraph label is given.
    ///
    /// # See also
    ///
    /// - [`get_sub_graph`](Self::get_sub_graph) for a fallible version.
    pub fn sub_graph(&self, label: impl RenderSubGraph) -> &SetupGraph {
        let label = label.intern();
        self.sub_graphs
            .get(&label)
            .unwrap_or_else(|| panic!("Subgraph {label:?} not found"))
    }

    /// Retrieves the sub graph corresponding to the `label` mutably.
    ///
    /// # Panics
    ///
    /// Panics if any invalid subgraph label is given.
    ///
    /// # See also
    ///
    /// - [`get_sub_graph_mut`](Self::get_sub_graph_mut) for a fallible version.
    pub fn sub_graph_mut(&mut self, label: impl RenderSubGraph) -> &mut SetupGraph {
        let label = label.intern();
        self.sub_graphs
            .get_mut(&label)
            .unwrap_or_else(|| panic!("Subgraph {label:?} not found"))
    }
}

impl Debug for SetupGraph {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for node in self.iter_nodes() {
            writeln!(f, "{:?}", node.label)?;
            writeln!(f, "  in: {:?}", node.input_slots)?;
            writeln!(f, "  out: {:?}", node.output_slots)?;
        }

        Ok(())
    }
}

pub struct GraphInputSetup {
    inputs: Vec<SlotInfo>,
}

impl Setup for GraphInputSetup {
    fn input(&self) -> Vec<SlotInfo> {
        self.inputs.clone()
    }

    fn output(&self) -> Vec<SlotInfo> {
        self.inputs.clone()
    }

    fn run(
        &self,
        graph: &mut SetupGraphContext,
        _frame_graph: &mut FrameGraph,
        _world: &World,
    ) -> Result<(), SetupRunError> {
        for i in 0..graph.inputs().len() {
            let input = graph.inputs()[i].clone();
            graph.set_output(i, input)?;
        }
        Ok(())
    }
}
