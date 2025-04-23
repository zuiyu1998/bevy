use super::{
    FrameGraph, GpuRead, GpuWrite, RenderContext, ResourceNode, ResourceNodeHandle, ResourceRef,
    Result, TypeHandle, VirtualResource,
};

pub struct PassNode {
    pub name: String,
    pub handle: TypeHandle<PassNode>,
    pub writes: Vec<TypeHandle<ResourceNode>>,
    pub reads: Vec<TypeHandle<ResourceNode>>,
    pub resource_request_array: Vec<TypeHandle<VirtualResource>>,
    pub resource_release_array: Vec<TypeHandle<VirtualResource>>,
    pub pass_data: Option<Box<dyn PassData>>,
}

impl PassNode {
    pub fn new(name: &str, handle: TypeHandle<PassNode>) -> Self {
        Self {
            name: name.to_string(),
            handle,
            writes: Default::default(),
            reads: Default::default(),
            resource_request_array: Default::default(),
            resource_release_array: Default::default(),
            pass_data: None,
        }
    }

    pub fn write<ResourceType>(
        &mut self,
        graph: &mut FrameGraph,
        resource_node_handle: ResourceNodeHandle<ResourceType>,
    ) -> ResourceRef<ResourceType, GpuWrite> {
        let resource = &mut graph.resources[resource_node_handle.resource_handle.index];
        resource.info.new_version();

        let resource_info = resource.info.clone();
        let new_resource_node_handle = graph.create_resource_node(resource_info);
        let new_resource_node = &mut graph.resource_nodes[new_resource_node_handle.index];
        new_resource_node.pass_node_writer = Some(self.handle);

        self.writes.push(new_resource_node_handle);

        ResourceRef::new(resource_node_handle.resource_handle)
    }

    pub fn read_from_board<ResourceType>(
        &mut self,
        graph: &FrameGraph,
        name: &str,
    ) -> Option<ResourceRef<ResourceType, GpuRead>> {
        if let Some(handle) = graph.resource_board.get(name) {
            if !self.reads.contains(&handle.resource_node_handle) {
                self.reads.push(handle.resource_node_handle);
            }

            Some(ResourceRef::new(handle.resource_handle))
        } else {
            None
        }
    }

    pub fn read<ResourceType>(
        &mut self,
        _graph: &FrameGraph,
        resource_node_handle: ResourceNodeHandle<ResourceType>,
    ) -> ResourceRef<ResourceType, GpuRead> {
        let handle = resource_node_handle.resource_node_handle;

        if !self.reads.contains(&handle) {
            self.reads.push(handle);
        }

        ResourceRef::new(resource_node_handle.resource_handle)
    }
}

pub trait PassData: 'static + Send + Sync {
    fn execute(&self, render_context: &mut RenderContext) -> Result<()>;
}
