use alloc::sync::Arc;

use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, COPY_BUFFER_ALIGNMENT,
    util::BufferInitDescriptor,
};

use super::{
    AnyArcTransientResource, AnyTransientResource, AnyTransientResourceDescriptor,
    IntoAnyArcTransientResource, TransientResource, TransientResourceDescriptor,
};

impl IntoAnyArcTransientResource for TransientBuffer {
    fn into_arc_transient_resource(self: Arc<Self>) -> AnyArcTransientResource {
        AnyArcTransientResource::Buffer(self)
    }
}

#[derive(Clone)]
pub struct TransientBuffer {
    pub resource: Buffer,
    pub desc: TransientBufferDescriptor,
}

impl TransientResource for TransientBuffer {
    type Descriptor = TransientBufferDescriptor;

    fn borrow_resource(res: &AnyTransientResource) -> &Self {
        match res {
            AnyTransientResource::OwnedBuffer(res) => res,
            AnyTransientResource::ImportedBuffer(res) => res,
            _ => {
                unimplemented!()
            }
        }
    }

    fn get_desc(&self) -> &Self::Descriptor {
        &self.desc
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct TransientBufferDescriptor {
    pub label: Option<String>,
    pub size: BufferAddress,
    pub usage: BufferUsages,
    pub mapped_at_creation: bool,
}

//todo update bevy
impl Default for TransientBufferDescriptor {
    fn default() -> Self {
        TransientBufferDescriptor {
            label: None,
            size: 0,
            usage: BufferUsages::all(),
            mapped_at_creation: false,
        }
    }
}

impl TransientBufferDescriptor {
    pub fn from_init_desc(desc: &BufferInitDescriptor) -> Self {
        if desc.contents.is_empty() {
            TransientBufferDescriptor {
                label: desc.label.as_ref().map(ToString::to_string),
                size: 0,
                usage: desc.usage,
                mapped_at_creation: false,
            }
        } else {
            let unpadded_size = desc.contents.len() as BufferAddress;
            // Valid vulkan usage is
            // 1. buffer size must be a multiple of COPY_BUFFER_ALIGNMENT.
            // 2. buffer size must be greater than 0.
            // Therefore we round the value up to the nearest multiple, and ensure it's at least COPY_BUFFER_ALIGNMENT.
            let align_mask = COPY_BUFFER_ALIGNMENT - 1;
            let padded_size =
                ((unpadded_size + align_mask) & !align_mask).max(COPY_BUFFER_ALIGNMENT);

            TransientBufferDescriptor {
                label: desc.label.as_ref().map(ToString::to_string),
                size: padded_size,
                usage: desc.usage,
                mapped_at_creation: false,
            }
        }
    }

    pub fn from_desc(desc: &BufferDescriptor) -> Self {
        Self {
            label: desc.label.as_ref().map(ToString::to_string),
            size: desc.size,
            usage: desc.usage,
            mapped_at_creation: desc.mapped_at_creation,
        }
    }

    pub fn get_desc(&self) -> BufferDescriptor<'_> {
        BufferDescriptor {
            label: self.label.as_deref(),
            size: self.size,
            usage: self.usage,
            mapped_at_creation: self.mapped_at_creation,
        }
    }
}

impl From<TransientBufferDescriptor> for AnyTransientResourceDescriptor {
    fn from(value: TransientBufferDescriptor) -> Self {
        AnyTransientResourceDescriptor::Buffer(value)
    }
}

impl TransientResourceDescriptor for TransientBufferDescriptor {
    type Resource = TransientBuffer;

    fn borrow_resource_descriptor(res: &AnyTransientResourceDescriptor) -> &Self {
        match res {
            AnyTransientResourceDescriptor::Buffer(res) => res,
            _ => {
                unimplemented!()
            }
        }
    }
}
