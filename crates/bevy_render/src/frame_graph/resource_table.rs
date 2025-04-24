use bevy_platform::collections::HashMap;

use crate::renderer::RenderDevice;

use super::{
    AnyResource, DeviceTrait, GpuRead, ImportedVirtualResource, Resource, ResourceRef,
    ResourceState, Result, TransientResourceCache, TypeHandle, VirtualResource,
};

#[derive(Default)]
pub struct ResourceTable {
    resources: HashMap<TypeHandle<VirtualResource>, AnyResource>,
}

impl ResourceTable {
    pub fn get_resource<ResourceType: Resource>(
        &self,
        resource_ref: &ResourceRef<ResourceType, GpuRead>,
    ) -> Option<&ResourceType> {
        self.resources
            .get(&resource_ref.index)
            .map(|any| ResourceType::borrow_resource(any))
    }

    pub fn request_resource(
        &mut self,
        resource: &VirtualResource,
        device: &RenderDevice,
        transient_resource_cache: &mut TransientResourceCache,
    ) {
        let handle = resource.info.handle;
        let resource = match &resource.state {
            ResourceState::Imported(state) => match &state.resource {
                ImportedVirtualResource::Texture(resource) => {
                    AnyResource::ImportedTexture(resource.clone())
                }
                ImportedVirtualResource::Buffer(resource) => {
                    AnyResource::ImportedBuffer(resource.clone())
                }
            },
            ResourceState::Setuped(desc) => transient_resource_cache
                .get_resource(&desc)
                .unwrap_or_else(|| device.create(desc)),
        };

        self.resources.insert(handle, resource);
    }

    pub fn release_resource(
        &mut self,
        handle: &TypeHandle<VirtualResource>,
        transient_resource_cache: &mut TransientResourceCache,
    ) {
        if let Some(resource) = self.resources.remove(handle) {
            match resource {
                AnyResource::OwnedBuffer(buffer) => {
                    transient_resource_cache.insert_resource(
                        buffer.desc.clone().into(),
                        AnyResource::OwnedBuffer(buffer),
                    );
                }
                AnyResource::OwnedTexture(texture) => {
                    transient_resource_cache.insert_resource(
                        texture.desc.clone().into(),
                        AnyResource::OwnedTexture(texture),
                    );
                }
                _ => {}
            }
        }
    }
}

pub trait ResourceView<'a>: Sized {
    type ViewRef: 'static;

    fn prepare_view(resource_table: &'a ResourceTable, view_ref: &'a Self::ViewRef) -> Result<Self>;
}
