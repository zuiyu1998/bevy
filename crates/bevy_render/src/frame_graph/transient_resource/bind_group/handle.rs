use core::num::NonZero;

use wgpu::{BindGroupLayout, Sampler};

use crate::frame_graph::{
    ResourceHandle, TransientBuffer, TransientTexture, TransientTextureViewDescriptor,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransientBindGroupTextureViewHandle {
    pub texture: ResourceHandle<TransientTexture>,
    pub texture_view_desc: TransientTextureViewDescriptor,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransientBindGroupBufferHandle {
    pub buffer: ResourceHandle<TransientBuffer>,
    pub size: Option<NonZero<u64>>,
    pub offset: u64,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TransientBindGroupResourceHandle {
    Buffer(TransientBindGroupBufferHandle),
    Sampler(Sampler),
    TextureView(TransientBindGroupTextureViewHandle),
    TextureViewArray(Vec<TransientBindGroupTextureViewHandle>),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransientBindGroupEntryHandle {
    pub binding: u32,
    pub resource: TransientBindGroupResourceHandle,
}

#[derive(Clone, PartialEq, Debug)]
pub struct TransientBindGroupHandle {
    pub label: Option<String>,
    pub layout: BindGroupLayout,
    pub entries: Vec<TransientBindGroupEntryHandle>,
}

impl TransientBindGroupBufferHandle {
    pub fn build(layout: BindGroupLayout) -> TransientBindGroupHandleBuilder {
        TransientBindGroupHandleBuilder::new(None, layout)
    }
}

pub trait IntoTransientBindGroupResourceHandle {
    fn into_handle(self) -> TransientBindGroupResourceHandle;
}

pub struct TransientBindGroupHandleBuilder {
    label: Option<String>,
    layout: BindGroupLayout,
    entries: Vec<TransientBindGroupEntryHandle>,
}

impl TransientBindGroupHandleBuilder {
    pub fn new(label: Option<String>, layout: BindGroupLayout) -> Self {
        Self {
            label,
            layout,
            entries: vec![],
        }
    }

    pub fn push<T: IntoTransientBindGroupResourceHandle>(mut self, value: T) -> Self {
        let handle = value.into_handle();

        self.entries.push(TransientBindGroupEntryHandle {
            resource: handle,
            binding: self.entries.len() as u32,
        });

        self
    }

    pub fn set_label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());

        self
    }

    pub fn finished(self) -> TransientBindGroupHandle {
        TransientBindGroupHandle {
            label: self.label,
            layout: self.layout,
            entries: self.entries,
        }
    }
}
