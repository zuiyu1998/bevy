use std::collections::HashMap;

use wgpu::Device;

use crate::{
    AnyTransientResource, ArcAnyTransientResource, IndexHandle, ResourceNode, ResourceRef,
    ResourceRelease, ResourceRequese, ResourceView, TransientResource, TransientResourceCache,
    TransientResourceCreator, VirtualResource,
};

#[derive(Default)]
pub struct ResourceTable {
    resources: HashMap<IndexHandle<ResourceNode>, AnyTransientResource>,
}

impl ResourceTable {
    pub fn get_resource<ResourceType: TransientResource, ViewType: ResourceView>(
        &self,
        resource_ref: &ResourceRef<ResourceType, ViewType>,
    ) -> &ResourceType {
        self.resources
            .get(&resource_ref.raw.index)
            .map(|res| TransientResource::borrow_resource(res))
            .expect("must have resource")
    }

    pub fn request_resource(
        &mut self,
        request: &ResourceRequese,
        device: &Device,
        transient_resource_cache: &mut TransientResourceCache,
    ) {
        let index = request.index;
        let resource = match &request.resource {
            VirtualResource::Imported(resource) => match &resource {
                ArcAnyTransientResource::Texture(resource) => {
                    AnyTransientResource::ImportedTexture(resource.clone())
                }
                ArcAnyTransientResource::Buffer(resource) => {
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
        if let Some(resource) = self.resources.remove(&release.index) {
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
