use alloc::sync::Arc;
use core::ops::Deref;

use bevy_render::render_resource::{Buffer, BufferSlice};

use crate::frame_graph::{
    FrameGraph, ResourceHandle, ResourceMaterial, TransientBuffer, TransientBufferDescriptor,
};

impl ResourceMaterial for Buffer {
    type ResourceType = TransientBuffer;

    fn imported(&self, frame_graph: &mut FrameGraph) -> ResourceHandle<Self::ResourceType> {
        let buffer_key = format!("{:?}", self.id());

        let buffer = TransientBuffer {
            resource: self.deref().clone(),
            desc: TransientBufferDescriptor::default(),
        };

        frame_graph.import(&buffer_key, Arc::new(buffer))
    }
}

impl<'a> ResourceMaterial for BufferSlice<'a> {
    type ResourceType = TransientBuffer;

    fn imported(&self, frame_graph: &mut FrameGraph) -> ResourceHandle<Self::ResourceType> {
        let buffer_key = format!("{:?}", self.id());

        let buffer = TransientBuffer {
            resource: self.buffer().clone(),
            desc: TransientBufferDescriptor::default(),
        };

        frame_graph.import(&buffer_key, Arc::new(buffer))
    }
}
