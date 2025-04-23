use std::sync::Arc;

use crate::frame_graph::{FrameGraphTexture, TextureInfo};

use super::{
    AnyResource, AnyResourceDescriptor, ImportToFrameGraph, ImportedVirtualResource, Resource,
    ResourceDescriptor,
};

impl Resource for FrameGraphTexture {
    type Descriptor = TextureInfo;

    fn borrow_resource(res: &AnyResource) -> &Self {
        match res {
            AnyResource::ImportedTexture(res) => res,
            AnyResource::OwnedTexture(res) => res,
            _ => {
                unimplemented!()
            }
        }
    }
}

impl ImportToFrameGraph for FrameGraphTexture {
    fn import(self: Arc<Self>) -> ImportedVirtualResource {
        ImportedVirtualResource::Texture(self)
    }
}

impl ResourceDescriptor for TextureInfo {
    type Resource = FrameGraphTexture;
}

impl From<TextureInfo> for AnyResourceDescriptor {
    fn from(value: TextureInfo) -> Self {
        AnyResourceDescriptor::Texture(value)
    }
}
