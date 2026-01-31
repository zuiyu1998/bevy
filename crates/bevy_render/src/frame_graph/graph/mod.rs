mod device_pass;
mod pass_node_builder;
mod resource_board;

pub use device_pass::*;
pub use pass_node_builder::*;
pub use resource_board::*;

use std::sync::Arc;
use wgpu::CommandBuffer;

use crate::{
    frame_graph::{
        IndexHandle, IntoAnyArcTransientResource, PassNode, PipelineContainer, ResourceHandle,
        ResourceNode, ResourceTable, TransientResource, TransientResourceCache,
        TransientResourceDescriptor, TypeEquals, VirtualResource,
    },
    renderer::RenderDevice,
};

pub struct FrameGraphContext<'a> {
    pub resource_table: ResourceTable,
    pub pipeline_container: PipelineContainer,
    pub render_device: RenderDevice,
    pub transient_resource_cache: &'a mut TransientResourceCache,
    pub(crate) command_buffers: Vec<CommandBuffer>,
}

impl<'a> FrameGraphContext<'a> {
    pub fn new(
        pipeline_container: PipelineContainer,
        render_device: &'a RenderDevice,
        transient_resource_cache: &'a mut TransientResourceCache,
    ) -> Self {
        Self {
            resource_table: Default::default(),
            pipeline_container,
            render_device: render_device.clone(),
            transient_resource_cache,
            command_buffers: vec![],
        }
    }

    pub fn add_command_buffer(&mut self, command_buffer: CommandBuffer) {
        self.command_buffers.push(command_buffer);
    }

    pub fn finish(self) -> Vec<CommandBuffer> {
        self.command_buffers
    }
}

pub trait ResourceMaterial {
    type ResourceType: TransientResource;

    fn imported(&self, frame_graph: &mut FrameGraph) -> ResourceHandle<Self::ResourceType>;
}

pub struct CompiledFrameGraph {
    device_passes: Vec<DevicePass>,
}

impl CompiledFrameGraph {
    pub fn execute(&self, context: &mut FrameGraphContext) {
        for device_pass in self.device_passes.iter() {
            device_pass.execute(context);
        }
    }
}

#[derive(Default)]
pub struct FrameGraph {
    pub(crate) resource_nodes: Vec<ResourceNode>,
    pub(crate) pass_nodes: Vec<PassNode>,
    pub(crate) compiled_frame_graph: Option<CompiledFrameGraph>,
    pub(crate) resource_board: ResourceBoard,
}

impl FrameGraph {
    pub fn reset(&mut self) {
        self.pass_nodes = vec![];
        self.resource_nodes = vec![];
        self.compiled_frame_graph = None;
        self.resource_board = ResourceBoard::default();
    }

    pub fn execute(&mut self, context: &mut FrameGraphContext) {
        if self.compiled_frame_graph.is_none() {
            return;
        }

        if let Some(compiled_frame_graph) = &mut self.compiled_frame_graph {
            compiled_frame_graph.execute(context);
        }

        self.reset();
    }

    fn compute_resource_lifetime(&mut self) {
        for pass_node in self.pass_nodes.iter_mut() {
            for resource_handle in pass_node.reads.iter() {
                let resource_node = &mut self.resource_nodes[resource_handle.index.index];
                resource_node.update_lifetime(pass_node.index);
            }

            for resource_handle in pass_node.writes.iter() {
                let resource_node = &mut self.resource_nodes[resource_handle.index.index];
                resource_node.update_lifetime(pass_node.index);
            }
        }

        for resource_index in 0..self.resource_nodes.len() {
            let resource_node = &self.resource_nodes[resource_index];

            if resource_node.first_use_pass.is_none() || resource_node.last_user_pass.is_none() {
                continue;
            }

            let first_pass_node_handle = resource_node.first_use_pass.unwrap();
            let first_pass_node = &mut self.pass_nodes[first_pass_node_handle.index];
            first_pass_node
                .resource_request_array
                .push(resource_node.index);

            let last_pass_node_handle = resource_node.last_user_pass.unwrap();
            let last_pass_node = &mut self.pass_nodes[last_pass_node_handle.index];
            last_pass_node
                .resource_release_array
                .push(resource_node.index);
        }
    }

    fn generate_compiled_frame_graph(&mut self) {
        if self.pass_nodes.is_empty() {
            return;
        }

        let mut device_passes = vec![];

        for index in 0..self.pass_nodes.len() {
            let type_index = self.pass_nodes[index].index;

            let mut device_pass = DevicePass::default();
            device_pass.extra(self, type_index);

            device_passes.push(device_pass);
        }

        self.compiled_frame_graph = Some(CompiledFrameGraph { device_passes });
    }

    pub fn compile(&mut self) {
        if self.pass_nodes.is_empty() {
            return;
        }
        //todo cull

        self.compute_resource_lifetime();
        self.generate_compiled_frame_graph();
    }
}

impl FrameGraph {
    pub fn create_pass_node_builder(&mut self, name: &str) -> PassNodeBuilder<'_> {
        PassNodeBuilder::new(name, self)
    }

    pub fn create_pass_buidlder(&mut self, name: &str) -> PassBuilder<'_> {
        PassBuilder::new(self.create_pass_node_builder(name))
    }

    pub fn insert(&mut self, key: &str, index: IndexHandle<ResourceNode>) {
        let key = key.into();
        self.resource_board.insert(key, index);
    }

    pub fn get<ResourceType: TransientResource>(
        &self,
        key: &str,
    ) -> Option<ResourceHandle<ResourceType>> {
        let key = key.into();

        self.resource_board
            .get(&key)
            .map(|handle| self.resource_nodes[handle.index].get_handle())
    }

    pub fn pass_node(&mut self, name: &str) -> &mut PassNode {
        let handle = IndexHandle::new(self.pass_nodes.len());
        let pass_node = PassNode::new(name, handle);
        self.pass_nodes.push(pass_node);

        self.get_pass_node_mut(&handle)
    }

    pub fn get_pass_node_mut(&mut self, handle: &IndexHandle<PassNode>) -> &mut PassNode {
        &mut self.pass_nodes[handle.index]
    }

    pub fn get_pass_node(&self, handle: &IndexHandle<PassNode>) -> &PassNode {
        &self.pass_nodes[handle.index]
    }

    pub fn get_resource_node_mut(
        &mut self,
        handle: &IndexHandle<ResourceNode>,
    ) -> &mut ResourceNode {
        &mut self.resource_nodes[handle.index]
    }

    pub fn get_resource_node(&self, handle: &IndexHandle<ResourceNode>) -> &ResourceNode {
        &self.resource_nodes[handle.index]
    }

    pub fn import<ResourceType>(
        &mut self,
        name: &str,
        resource: Arc<ResourceType>,
    ) -> ResourceHandle<ResourceType>
    where
        ResourceType: IntoAnyArcTransientResource,
    {
        let key = name.into();
        if let Some(raw_handle) = self.resource_board.get(&key) {
            return self.resource_nodes[raw_handle.index].get_handle();
        }

        let resource_node_handle = IndexHandle::new(self.resource_nodes.len());
        let virtual_resource = VirtualResource::Imported(
            IntoAnyArcTransientResource::into_arc_transient_resource(resource),
        );
        let resource_node = ResourceNode::new(name, resource_node_handle, virtual_resource);

        let handle = resource_node.get_handle();

        self.resource_nodes.push(resource_node);

        self.insert(name, handle.raw.index);

        handle
    }

    pub fn get_or_create<DescriptorType>(&mut self, name: &str, desc: DescriptorType) -> ResourceHandle<DescriptorType::Resource>
    where
        DescriptorType: TransientResourceDescriptor
            + TypeEquals<
                Other = <<DescriptorType as TransientResourceDescriptor>::Resource as TransientResource>::Descriptor,
            >,
    {
        let key = name.into();
        if let Some(raw_handle) = self.resource_board.get(&key) {
            return self.resource_nodes[raw_handle.index].get_handle();
        }

        let handle = self.create(name, desc);

        self.resource_board.insert(key, handle.raw.index);

        handle
    }

    pub fn create<DescriptorType>(&mut self, name: &str, desc: DescriptorType) -> ResourceHandle<DescriptorType::Resource>
    where
        DescriptorType: TransientResourceDescriptor
            + TypeEquals<
                Other = <<DescriptorType as TransientResourceDescriptor>::Resource as TransientResource>::Descriptor,
            >,
    {
        let resource_node_handle = IndexHandle::new(self.resource_nodes.len());
        let virtual_resource = VirtualResource::Setuped(desc.into());
        let resource_node = ResourceNode::new(name, resource_node_handle, virtual_resource);

        let handle = resource_node.get_handle();

        self.resource_nodes.push(resource_node);

        handle
    }
}
