mod device_pass;

pub use device_pass::*;

use crate::{
    frame_graph::{
        Handle, IndexHandle, IntoArcTransientResource, PassNode, ResourceBoard, ResourceNode,
        ResourceTable, TransientResource, TransientResourceCache, TransientResourceDescriptor,
        TypeEquals, VirtualResource,
    },
    renderer::RenderDevice,
};
use alloc::sync::Arc;
use bevy_ecs::component::Component;
use wgpu::CommandBuffer;

pub struct PipelineContainer {}

pub trait GetPipelineContainer {
    fn get_pipeline_container(&self) -> PipelineContainer;
}

pub struct FrameGraphContext<'a> {
    pub resource_table: ResourceTable,
    pub pipeline_container: PipelineContainer,
    pub render_device: RenderDevice,
    pub transient_resource_cache: &'a mut TransientResourceCache,
    command_buffers: Vec<CommandBuffer>,
}

impl<'a> FrameGraphContext<'a> {
    pub fn new<T: GetPipelineContainer>(
        pipeline_cache: &'a T,
        render_device: &'a RenderDevice,
        transient_resource_cache: &'a mut TransientResourceCache,
    ) -> Self {
        Self {
            resource_table: Default::default(),
            pipeline_container: pipeline_cache.get_pipeline_container(),
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

    fn imported(&self, frame_graph: &mut FrameGraph) -> Handle<Self::ResourceType>;
}

pub struct CompiledFrameGraph {
    device_passes: Vec<DevicePass>,
}

impl CompiledFrameGraph {
    pub fn execute(&self, ctx: &mut FrameGraphContext) {
        for device_pass in self.device_passes.iter() {
            device_pass.execute(ctx);
        }
    }
}

#[derive(Default, Component)]
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

    pub fn execute(&mut self, ctx: &mut FrameGraphContext) {
        if self.compiled_frame_graph.is_none() {
            return;
        }

        if let Some(compiled_frame_graph) = &mut self.compiled_frame_graph {
            compiled_frame_graph.execute(ctx);
        }

        self.reset();
    }

    pub fn compute_resource_lifetime(&mut self) {
        for pass_node in self.pass_nodes.iter_mut() {
            for resource_handle in pass_node.reads.iter() {
                let resource_node = &mut self.resource_nodes[resource_handle.index()];
                resource_node.update_lifetime(pass_node.handle.clone());
            }

            for resource_handle in pass_node.writes.iter() {
                let resource_node = &mut self.resource_nodes[resource_handle.index()];
                resource_node.update_lifetime(pass_node.handle.clone());
            }
        }

        for resource_index in 0..self.resource_nodes.len() {
            let resource_node = &self.resource_nodes[resource_index];

            if resource_node.first_use_pass.is_none() || resource_node.last_user_pass.is_none() {
                continue;
            }

            let first_pass_node_handle = resource_node.first_use_pass.clone().unwrap();
            let first_pass_node = &mut self.pass_nodes[first_pass_node_handle.index()];
            first_pass_node
                .resource_request_array
                .push(resource_node.handle.clone());

            let last_pass_node_handle = resource_node.last_user_pass.clone().unwrap();
            let last_pass_node = &mut self.pass_nodes[last_pass_node_handle.index()];
            last_pass_node
                .resource_release_array
                .push(resource_node.handle.clone());
        }
    }

    pub fn generate_compiled_frame_graph(&mut self) {
        if self.pass_nodes.is_empty() {
            return;
        }

        let mut device_passes = vec![];

        for index in 0..self.pass_nodes.len() {
            let handle = self.pass_nodes[index].handle.clone();

            let mut device_pass = DevicePass::default();
            device_pass.extra(self, &handle);

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
    pub fn insert(&mut self, key: &str, index: IndexHandle<ResourceNode>) {
        let key = key.into();
        self.resource_board.insert(key, index);
    }

    pub fn get<ResourceType: TransientResource>(&self, key: &str) -> Option<Handle<ResourceType>> {
        let key = key.into();

        self.resource_board
            .get(&key)
            .map(|handle| self.resource_nodes[handle.index()].get_handle())
    }

    pub fn pass_node(&mut self, name: &str) -> &mut PassNode {
        let handle = IndexHandle::new(self.pass_nodes.len());
        let pass_node = PassNode::new(name, handle.clone());
        self.pass_nodes.push(pass_node);

        self.get_pass_node_mut(&handle)
    }

    pub fn get_pass_node_mut(&mut self, handle: &IndexHandle<PassNode>) -> &mut PassNode {
        &mut self.pass_nodes[handle.index()]
    }

    pub fn get_pass_node(&self, handle: &IndexHandle<PassNode>) -> &PassNode {
        &self.pass_nodes[handle.index()]
    }

    pub fn get_resource_node_mut(
        &mut self,
        handle: &IndexHandle<ResourceNode>,
    ) -> &mut ResourceNode {
        &mut self.resource_nodes[handle.index()]
    }

    pub fn get_resource_node(&self, handle: &IndexHandle<ResourceNode>) -> &ResourceNode {
        &self.resource_nodes[handle.index()]
    }

    pub fn import<ResourceType>(
        &mut self,
        name: &str,
        resource: Arc<ResourceType>,
    ) -> Handle<ResourceType>
    where
        ResourceType: IntoArcTransientResource,
    {
        let key = name.into();
        if let Some(raw_handle) = self.resource_board.get(&key) {
            return self.resource_nodes[raw_handle.index()].get_handle();
        }

        let resource_node_handle = IndexHandle::new(self.resource_nodes.len());
        let virtual_resource = VirtualResource::Imported(
            IntoArcTransientResource::into_arc_transient_resource(resource),
        );
        let resource_node = ResourceNode::new(name, resource_node_handle, virtual_resource);

        let handle = resource_node.get_handle();

        self.resource_nodes.push(resource_node);

        self.insert(name, handle.get_resource_handle().handle().clone());

        handle
    }

    pub fn get_or_create<DescriptorType>(&mut self, name: &str, desc: DescriptorType) -> Handle<DescriptorType::Resource>
    where
        DescriptorType: TransientResourceDescriptor
            + TypeEquals<
                Other = <<DescriptorType as TransientResourceDescriptor>::Resource as TransientResource>::Descriptor,
            >,
    {
        let key = name.into();
        if let Some(raw_handle) = self.resource_board.get(&key) {
            return self.resource_nodes[raw_handle.index()].get_handle();
        }

        let handle = self.create(name, desc);

        self.resource_board
            .insert(key, handle.get_resource_handle().handle().clone());

        handle
    }

    pub fn create<DescriptorType>(&mut self, name: &str, desc: DescriptorType) -> Handle<DescriptorType::Resource>
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
