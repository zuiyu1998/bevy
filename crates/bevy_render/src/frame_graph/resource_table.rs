use bevy_platform::collections::HashMap;

use crate::{
    frame_graph::{
        AnyTransientResource, ArcTransientResource, ResourceRelease, ResourceRequese,
        TransientResource, TransientResourceCache, TransientResourceCreator, VirtualResource,
    },
    renderer::RenderDevice,
};

pub struct ResourceTable {
    resources: HashMap<usize, AnyTransientResource>,
}

impl Default for ResourceTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceTable {
    pub fn new() -> Self {
        Self {
            resources: Default::default(),
        }
    }

    pub fn insert(&mut self, index: usize, resource: AnyTransientResource) {
        self.resources.insert(index, resource);
    }

    pub fn get_resource<ResourceType: TransientResource>(&self, index: &usize) -> &ResourceType {
        self.try_get_resource(index).expect("Resource not found")
    }

    pub fn try_get_resource<ResourceType: TransientResource>(
        &self,
        index: &usize,
    ) -> Option<&ResourceType> {
        Some(ResourceType::borrow_resource(self.resources.get(index)?))
    }

    pub fn get(&self, index: &usize) -> Option<&AnyTransientResource> {
        self.resources.get(index)
    }

    pub fn request_resource(
        &mut self,
        request: &ResourceRequese,
        device: &RenderDevice,
        transient_resource_cache: &mut TransientResourceCache,
    ) {
        let index = request.handle.index();
        let resource = match &request.resource {
            VirtualResource::Imported(resource) => match &resource {
                ArcTransientResource::Texture(resource) => {
                    AnyTransientResource::ImportedTexture(resource.clone())
                }
                ArcTransientResource::Buffer(resource) => {
                    AnyTransientResource::ImportedBuffer(resource.clone())
                }
            },
            VirtualResource::Setuped(desc) => transient_resource_cache
                .get_resource(desc)
                .unwrap_or_else(|| device.create_resource(desc)),
        };

        self.resources.insert(index, resource);
    }

    pub fn release_resource(
        &mut self,
        release: &ResourceRelease,
        transient_resource_cache: &mut TransientResourceCache,
    ) {
        let index = release.handle.index();

        if let Some(resource) = self.resources.remove(&index) {
            match resource {
                AnyTransientResource::OwnedBuffer(buffer) => {
                    transient_resource_cache.insert_resource(
                        buffer.desc.clone().into(),
                        AnyTransientResource::OwnedBuffer(buffer),
                    );
                }
                AnyTransientResource::OwnedTexture(texture) => {
                    transient_resource_cache.insert_resource(
                        texture.desc.clone().into(),
                        AnyTransientResource::OwnedTexture(texture),
                    );
                }
                _ => {}
            }
        }
    }
}
