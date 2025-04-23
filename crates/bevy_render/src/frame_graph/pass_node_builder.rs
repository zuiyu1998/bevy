use std::sync::Arc;

use super::{
    FrameGraph, GpuRead, GpuWrite, ImportToFrameGraph, PassData, PassNode, Resource,
    ResourceDescriptor, ResourceNodeHandle, ResourceRef, TypeEquals, TypeHandle,
};

pub struct PassNodeBuilder<'a> {
    graph: &'a mut FrameGraph,
    pass_node: Option<PassNode>,
}

impl Drop for PassNodeBuilder<'_> {
    fn drop(&mut self) {
        self.build();
    }
}

impl<'a> PassNodeBuilder<'a> {
    pub fn set_pass_data<T>(&mut self, pass_data: T)
    where
        T: PassData,
    {
        self.pass_node.as_mut().unwrap().pass_data = Some(Box::new(pass_data));
    }

    pub fn new(name: &str, graph: &'a mut FrameGraph) -> Self {
        let handle = TypeHandle::new(graph.pass_nodes.len());
        Self {
            graph,
            pass_node: Some(PassNode::new(name, handle)),
        }
    }

    fn build(&mut self) {
        assert!(self.pass_node.as_ref().unwrap().pass_data.is_some());

        let pass_node = self.pass_node.take().unwrap();
        self.graph.pass_nodes.push(pass_node);
    }

    pub fn import<ResourceType>(
        &mut self,
        name: &str,
        resource: Arc<ResourceType>,
    ) -> ResourceNodeHandle<ResourceType>
    where
        ResourceType: ImportToFrameGraph,
    {
        let desc = resource.get_desc().clone();
        self.graph.import(name, resource, desc)
    }

    pub fn create<DescriptorType>(
        &mut self,
        name: &str,
        desc: DescriptorType,
    ) -> ResourceNodeHandle<DescriptorType::Resource>
    where
        DescriptorType: ResourceDescriptor
            + TypeEquals<
                Other = <<DescriptorType as ResourceDescriptor>::Resource as Resource>::Descriptor,
            >,
    {
        self.graph.create(name, desc)
    }

    pub fn read_from_board<ResourceType>(
        &mut self,
        name: &str,
    ) -> Option<ResourceRef<ResourceType, GpuRead>> {
        self.pass_node
            .as_mut()
            .unwrap()
            .read_from_board(self.graph, name)
    }

    pub fn read<ResourceType>(
        &mut self,
        resource_node_handle: ResourceNodeHandle<ResourceType>,
    ) -> ResourceRef<ResourceType, GpuRead> {
        self.pass_node
            .as_mut()
            .unwrap()
            .read(self.graph, resource_node_handle)
    }

    pub fn write<ResourceType>(
        &mut self,
        resource_node_handle: ResourceNodeHandle<ResourceType>,
    ) -> ResourceRef<ResourceType, GpuWrite> {
        self.pass_node
            .as_mut()
            .unwrap()
            .write(self.graph, resource_node_handle)
    }
}
