use core::ops::{Deref, DerefMut};

use crate::frame_graph::{
    FrameGraph, Handle, Pass, Ref, ResourceHandle, ResourceMaterial, ResourceRead, ResourceWrite,
    TransientResource, TypeEquals,
};

pub struct PassNodeBuilder<'a> {
    pub(crate) graph: &'a mut FrameGraph,
    pub(crate) name: String,
    writes: Vec<ResourceHandle>,
    reads: Vec<ResourceHandle>,
    pass: Option<Pass>,
}

impl Deref for PassNodeBuilder<'_> {
    type Target = FrameGraph;

    fn deref(&self) -> &Self::Target {
        self.graph
    }
}

impl DerefMut for PassNodeBuilder<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graph
    }
}

impl Drop for PassNodeBuilder<'_> {
    fn drop(&mut self) {
        let pass_node = self.graph.pass_node(&self.name);
        pass_node.writes = self.writes.clone();
        pass_node.reads = self.reads.clone();
        pass_node.pass = self.pass.take();
    }
}

impl<'a> PassNodeBuilder<'a> {
    pub fn set_pass(&mut self, mut pass: Pass) {
        pass.set_name(&self.name);
        self.pass = Some(pass)
    }

    pub fn read_material<M: ResourceMaterial>(
        &mut self,
        material: &M,
    ) -> Ref<M::ResourceType, ResourceRead> {
        let handle = material.imported(self.graph);
        self.read(handle)
    }

    pub fn write_material<M: ResourceMaterial>(
        &mut self,
        material: &M,
    ) -> Ref<M::ResourceType, ResourceWrite> {
        let handle = material.imported(self.graph);
        self.write(handle)
    }

    pub fn read<ResourceType: TransientResource>(
        &mut self,
        handle: Handle<ResourceType>,
    ) -> Ref<ResourceType, ResourceRead> {
        let resource_handle = handle.get_resource_handle().clone();
        let desc = TypeEquals::same(handle.get_desc().clone());

        if !self.reads.contains(&resource_handle) {
            self.reads.push(resource_handle.clone());
        }

        Ref::new(resource_handle, desc)
    }

    pub fn write<ResourceType: TransientResource>(
        &mut self,
        handle: Handle<ResourceType>,
    ) -> Ref<ResourceType, ResourceWrite> {
        let resource_handle = handle.get_resource_handle();
        let desc = TypeEquals::same(handle.get_desc().clone());

        let resource_node = &mut self.graph.get_resource_node_mut(resource_handle.handle());
        resource_node.new_version();

        let new_raw =
            ResourceHandle::new(resource_handle.handle().clone(), resource_node.version());

        self.writes.push(new_raw.clone());

        Ref::new(new_raw, desc)
    }

    pub fn new(name: &str, graph: &'a mut FrameGraph) -> Self {
        Self {
            graph,
            name: name.to_string(),
            writes: vec![],
            reads: vec![],
            pass: None,
        }
    }
}
