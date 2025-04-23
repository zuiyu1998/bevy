use std::sync::Arc;

use crate::frame_graph::{BufferInfo, FrameGraphBuffer};

use super::{
    AnyResource, AnyResourceDescriptor, ImportToFrameGraph, ImportedVirtualResource, Resource,
    ResourceDescriptor,
};

impl Resource for FrameGraphBuffer {
    type Descriptor = BufferInfo;

    fn borrow_resource(res: &AnyResource) -> &Self {
        match res {
            AnyResource::ImportedBuffer(res) => res,
            AnyResource::OwnedBuffer(res) => res,
            _ => {
                unimplemented!()
            }
        }
    }
}

impl ImportToFrameGraph for FrameGraphBuffer {
    fn import(self: Arc<Self>) -> ImportedVirtualResource {
        ImportedVirtualResource::Buffer(self)
    }
}

impl ResourceDescriptor for BufferInfo {
    type Resource = FrameGraphBuffer;
}

impl From<BufferInfo> for AnyResourceDescriptor {
    fn from(value: BufferInfo) -> Self {
        AnyResourceDescriptor::Buffer(value)
    }
}
