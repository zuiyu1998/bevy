use crate::{
    frame_graph::FrameGraph,
    render_graph::{InternedRenderLabel, InternedRenderSubGraph, RenderSubGraph},
};
pub use bevy_ecs::label::DynEq;
use bevy_ecs::{
    query::{QueryItem, QueryState, ReadOnlyQueryData},
    world::{FromWorld, World},
};
use core::fmt::Debug;
use downcast_rs::{impl_downcast, Downcast};
use thiserror::Error;

pub use bevy_render_macros::RenderLabel;

use super::{
    Edge, InputSlotError, OutputSlotError, RunSubGraphError, SetupGraphContext, SetupGraphError,
    SlotInfo, SlotInfos,
};

pub trait Setup: Downcast + Send + Sync + 'static {
    fn input(&self) -> Vec<SlotInfo> {
        Vec::new()
    }

    fn output(&self) -> Vec<SlotInfo> {
        Vec::new()
    }

    fn update(&mut self, _world: &mut World) {}

    fn run<'w>(
        &self,
        graph: &mut SetupGraphContext,
        frame_graph: &mut FrameGraph,
        world: &'w World,
    ) -> Result<(), SetupRunError>;
}

impl_downcast!(Setup);

#[derive(Error, Debug, Eq, PartialEq)]
pub enum SetupRunError {
    #[error("encountered an input slot error")]
    InputSlotError(#[from] InputSlotError),
    #[error("encountered an output slot error")]
    OutputSlotError(#[from] OutputSlotError),
    #[error("encountered an error when running a sub-graph")]
    RunSubGraphError(#[from] RunSubGraphError),
}

#[derive(Debug)]
pub struct Edges {
    label: InternedRenderLabel,
    input_edges: Vec<Edge>,
    output_edges: Vec<Edge>,
}

impl Edges {
    /// Returns all "input edges" (edges going "in") for this node .
    #[inline]
    pub fn input_edges(&self) -> &[Edge] {
        &self.input_edges
    }

    /// Returns all "output edges" (edges going "out") for this node .
    #[inline]
    pub fn output_edges(&self) -> &[Edge] {
        &self.output_edges
    }

    /// Returns this node's label.
    #[inline]
    pub fn label(&self) -> InternedRenderLabel {
        self.label
    }

    /// Adds an edge to the `input_edges` if it does not already exist.
    pub(crate) fn add_input_edge(&mut self, edge: Edge) -> Result<(), SetupGraphError> {
        if self.has_input_edge(&edge) {
            return Err(SetupGraphError::EdgeAlreadyExists(edge));
        }
        self.input_edges.push(edge);
        Ok(())
    }

    /// Removes an edge from the `input_edges` if it exists.
    pub(crate) fn remove_input_edge(&mut self, edge: Edge) -> Result<(), SetupGraphError> {
        if let Some(index) = self.input_edges.iter().position(|e| *e == edge) {
            self.input_edges.swap_remove(index);
            Ok(())
        } else {
            Err(SetupGraphError::EdgeDoesNotExist(edge))
        }
    }

    /// Adds an edge to the `output_edges` if it does not already exist.
    pub(crate) fn add_output_edge(&mut self, edge: Edge) -> Result<(), SetupGraphError> {
        if self.has_output_edge(&edge) {
            return Err(SetupGraphError::EdgeAlreadyExists(edge));
        }
        self.output_edges.push(edge);
        Ok(())
    }

    /// Removes an edge from the `output_edges` if it exists.
    pub(crate) fn remove_output_edge(&mut self, edge: Edge) -> Result<(), SetupGraphError> {
        if let Some(index) = self.output_edges.iter().position(|e| *e == edge) {
            self.output_edges.swap_remove(index);
            Ok(())
        } else {
            Err(SetupGraphError::EdgeDoesNotExist(edge))
        }
    }

    /// Checks whether the input edge already exists.
    pub fn has_input_edge(&self, edge: &Edge) -> bool {
        self.input_edges.contains(edge)
    }

    /// Checks whether the output edge already exists.
    pub fn has_output_edge(&self, edge: &Edge) -> bool {
        self.output_edges.contains(edge)
    }

    /// Searches the `input_edges` for a [`Edge::SlotEdge`],
    /// which `input_index` matches the `index`;
    pub fn get_input_slot_edge(&self, index: usize) -> Result<&Edge, SetupGraphError> {
        self.input_edges
            .iter()
            .find(|e| {
                if let Edge::SlotEdge { input_index, .. } = e {
                    *input_index == index
                } else {
                    false
                }
            })
            .ok_or(SetupGraphError::UnconnectedNodeInputSlot {
                input_slot: index,
                node: self.label,
            })
    }

    /// Searches the `output_edges` for a [`Edge::SlotEdge`],
    /// which `output_index` matches the `index`;
    pub fn get_output_slot_edge(&self, index: usize) -> Result<&Edge, SetupGraphError> {
        self.output_edges
            .iter()
            .find(|e| {
                if let Edge::SlotEdge { output_index, .. } = e {
                    *output_index == index
                } else {
                    false
                }
            })
            .ok_or(SetupGraphError::UnconnectedNodeOutputSlot {
                output_slot: index,
                node: self.label,
            })
    }
}

/// The internal representation of a [`Setup`], with all data required
/// by the [`RenderGraph`](super::RenderGraph).
///
/// The `input_slots` and `output_slots` are provided by the `node`.
pub struct NodeState {
    pub label: InternedRenderLabel,
    /// The name of the type that implements [`Setup`].
    pub type_name: &'static str,
    pub node: Box<dyn Setup>,
    pub input_slots: SlotInfos,
    pub output_slots: SlotInfos,
    pub edges: Edges,
}

impl Debug for NodeState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{:?} ({})", self.label, self.type_name)
    }
}

impl NodeState {
    /// Creates an [`NodeState`] without edges, but the `input_slots` and `output_slots`
    /// are provided by the `node`.
    pub fn new<T>(label: InternedRenderLabel, node: T) -> Self
    where
        T: Setup,
    {
        NodeState {
            label,
            input_slots: node.input().into(),
            output_slots: node.output().into(),
            node: Box::new(node),
            type_name: core::any::type_name::<T>(),
            edges: Edges {
                label,
                input_edges: Vec::new(),
                output_edges: Vec::new(),
            },
        }
    }

    /// Retrieves the [`Setup`].
    pub fn node<T>(&self) -> Result<&T, SetupGraphError>
    where
        T: Setup,
    {
        self.node
            .downcast_ref::<T>()
            .ok_or(SetupGraphError::WrongNodeType)
    }

    /// Retrieves the [`Setup`] mutably.
    pub fn node_mut<T>(&mut self) -> Result<&mut T, SetupGraphError>
    where
        T: Setup,
    {
        self.node
            .downcast_mut::<T>()
            .ok_or(SetupGraphError::WrongNodeType)
    }

    /// Validates that each input slot corresponds to an input edge.
    pub fn validate_input_slots(&self) -> Result<(), SetupGraphError> {
        for i in 0..self.input_slots.len() {
            self.edges.get_input_slot_edge(i)?;
        }

        Ok(())
    }

    /// Validates that each output slot corresponds to an output edge.
    pub fn validate_output_slots(&self) -> Result<(), SetupGraphError> {
        for i in 0..self.output_slots.len() {
            self.edges.get_output_slot_edge(i)?;
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct EmptyNode;

impl Setup for EmptyNode {
    fn run(
        &self,
        _graph: &mut SetupGraphContext,
        _frame_graph: &mut FrameGraph,
        _world: &World,
    ) -> Result<(), SetupRunError> {
        Ok(())
    }
}

/// A [`RenderGraph`](super::RenderGraph) [`Setup`] that runs the configured subgraph once.
/// This makes it easier to insert sub-graph runs into a graph.
pub struct RunGraphOnViewNode {
    sub_graph: InternedRenderSubGraph,
}

impl RunGraphOnViewNode {
    pub fn new<T: RenderSubGraph>(sub_graph: T) -> Self {
        Self {
            sub_graph: sub_graph.intern(),
        }
    }
}

impl Setup for RunGraphOnViewNode {
    fn run(
        &self,
        graph: &mut SetupGraphContext,
        _frame_graph: &mut FrameGraph,

        _world: &World,
    ) -> Result<(), SetupRunError> {
        graph.run_sub_graph(self.sub_graph, vec![], Some(graph.view_entity()))?;
        Ok(())
    }
}

pub trait ViewSetup {
    type ViewQuery: ReadOnlyQueryData;

    fn update(&mut self, _world: &mut World) {}

    fn run<'w>(
        &self,
        graph: &mut SetupGraphContext,
        frame_graph: &mut FrameGraph,
        view_query: QueryItem<'w, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), SetupRunError>;
}

pub struct ViewNodeRunner<N: ViewSetup> {
    view_query: QueryState<N::ViewQuery>,
    node: N,
}

impl<N: ViewSetup> ViewNodeRunner<N> {
    pub fn new(node: N, world: &mut World) -> Self {
        Self {
            view_query: world.query_filtered(),
            node,
        }
    }
}

impl<N: ViewSetup + FromWorld> FromWorld for ViewNodeRunner<N> {
    fn from_world(world: &mut World) -> Self {
        Self::new(N::from_world(world), world)
    }
}

impl<T> Setup for ViewNodeRunner<T>
where
    T: ViewSetup + Send + Sync + 'static,
{
    fn update(&mut self, world: &mut World) {
        self.view_query.update_archetypes(world);
        self.node.update(world);
    }

    fn run<'w>(
        &self,
        graph: &mut SetupGraphContext,
        frame_graph: &mut FrameGraph,
        world: &'w World,
    ) -> Result<(), SetupRunError> {
        let Ok(view) = self.view_query.get_manual(world, graph.view_entity()) else {
            return Ok(());
        };

        ViewSetup::run(&self.node, graph, frame_graph, view, world)?;
        Ok(())
    }
}
