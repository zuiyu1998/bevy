use std::sync::Arc;

use super::{
    DevicePass, ImportToFrameGraph, PassNode, RenderContext, Resource, ResourceBoard,
    ResourceDescriptor, ResourceInfo, ResourceNode, ResourceNodeHandle, TypeEquals, TypeHandle,
    VirtualResource,
};

pub struct FrameGraph {
    pub(crate) resources: Vec<VirtualResource>,
    pub(crate) resource_nodes: Vec<ResourceNode>,
    pub(crate) pass_nodes: Vec<PassNode>,
    device_passes: Option<Vec<DevicePass>>,
    pub(crate) resource_board: ResourceBoard,
}

impl FrameGraph {
    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    pub fn import<ResourceType>(
        &mut self,
        name: &str,
        resource: Arc<ResourceType>,
        desc: ResourceType::Descriptor,
    ) -> ResourceNodeHandle<ResourceType>
    where
        ResourceType: ImportToFrameGraph,
    {
        if let Some(raw_handle) = self.resource_board.get(name) {
            return ResourceNodeHandle::new(
                raw_handle.resource_node_handle,
                raw_handle.resource_handle,
            );
        }

        let imported_resource = ImportToFrameGraph::import(resource);
        let resource_handle = TypeHandle::new(self.resources.len());
        let resource: VirtualResource = VirtualResource::new_imported::<ResourceType>(
            name,
            resource_handle,
            desc,
            imported_resource,
        );

        let resource_info = resource.info.clone();
        self.resources.push(resource);

        let handle = self.create_resource_node(resource_info);

        let handle = ResourceNodeHandle::new(handle, resource_handle);

        self.resource_board.put(name, handle.raw());

        handle
    }

    pub(crate) fn create_resource_node(
        &mut self,
        resource_info: ResourceInfo,
    ) -> TypeHandle<ResourceNode> {
        let resource_handle = resource_info.handle;
        let version = resource_info.version();

        let handle = TypeHandle::new(self.resource_nodes.len());

        self.resource_nodes
            .push(ResourceNode::new(handle, resource_handle, version));

        handle
    }

    #[allow(unreachable_code)]
    #[allow(unused_variables)]
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
        let resource_handle = TypeHandle::new(self.resources.len());

        let virtual_resource: VirtualResource = {
            VirtualResource::new_setuped::<DescriptorType::Resource>(
                TypeEquals::same(desc),
                name,
                resource_handle,
            )
        };
        let resource_info = virtual_resource.info.clone();

        self.resources.push(virtual_resource);

        let handle = self.create_resource_node(resource_info);

        ResourceNodeHandle::new(handle, resource_handle)
    }
}

impl FrameGraph {
    fn reset(&mut self) {
        self.pass_nodes = vec![];
        self.resources = vec![];
        self.resource_nodes = vec![];
        self.resource_board = Default::default();
        self.device_passes = None;
    }

    pub fn execute(&mut self, render_context: &mut RenderContext) {
        if self.device_passes.is_none() {
            return;
        }

        let device_passes = self.device_passes.take().unwrap();

        for mut device_pass in device_passes {
            if let Err(e) = device_pass.execute(render_context) {
                panic!("render backend error: {}", e);
            }
        }

        self.reset();
    }

    pub fn compute_resource_lifetime(&mut self) {
        for pass_node in self.pass_nodes.iter_mut() {
            for resource_node_handle in pass_node.reads.iter() {
                let resource_node = &self.resource_nodes[resource_node_handle.index];
                let resource = &mut self.resources[resource_node.resource_handle.index];
                resource.info.update_lifetime(pass_node.handle);
            }

            for resource_node_handle in pass_node.writes.iter() {
                let resource_node = &self.resource_nodes[resource_node_handle.index];
                let resource = &mut self.resources[resource_node.resource_handle.index];
                resource.info.update_lifetime(pass_node.handle);
            }
        }

        for resource_index in 0..self.resources.len() {
            let resource = &self.resources[resource_index];
            let info = resource.info.clone();

            if info.first_use_pass.is_none() || info.last_user_pass.is_none() {
                continue;
            }

            let first_pass_node_handle = info.first_use_pass.unwrap();
            let first_pass_node = &mut self.pass_nodes[first_pass_node_handle.index];
            first_pass_node.resource_request_array.push(info.handle);

            let last_pass_node_handle = info.last_user_pass.unwrap();
            let last_pass_node = &mut self.pass_nodes[last_pass_node_handle.index];
            last_pass_node.resource_release_array.push(info.handle);
        }
    }

    fn generate_device_passes(&mut self) {
        if self.pass_nodes.is_empty() {
            return;
        }

        let mut device_passes = vec![];

        for index in 0..self.pass_nodes.len() {
            let pass_node_handle = TypeHandle::new(index);

            let mut device_pass = DevicePass::default();

            device_pass.extra(self, pass_node_handle);
            device_passes.push(device_pass);
        }

        self.device_passes = Some(device_passes);
    }

    pub fn compile(&mut self) {
        if self.pass_nodes.is_empty() {
            return;
        }

        //todo cull

        self.compute_resource_lifetime();

        self.generate_device_passes();
    }
}
