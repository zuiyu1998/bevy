use core::num::NonZero;

use wgpu::{BindGroupLayout, Sampler};

use crate::frame_graph::{
    PassNodeBuilderExt, ResourceHandle, TransientBindGroup, TransientBindGroupBuffer,
    TransientBindGroupEntry, TransientBindGroupResource, TransientBindGroupTextureView,
    TransientBuffer, TransientTexture, TransientTextureViewDescriptor,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransientBindGroupTextureViewHandle {
    pub texture: ResourceHandle<TransientTexture>,
    pub texture_view_desc: TransientTextureViewDescriptor,
}

impl TransientBindGroupTextureViewHandle {
    pub fn create<T: PassNodeBuilderExt>(
        &self,
        pass_node_builder: &mut T,
    ) -> TransientBindGroupTextureView {
        let texture = pass_node_builder.read(self.texture.clone());

        TransientBindGroupTextureView {
            texture,
            texture_view_desc: self.texture_view_desc.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransientBindGroupBufferHandle {
    pub buffer: ResourceHandle<TransientBuffer>,
    pub size: Option<NonZero<u64>>,
    pub offset: u64,
}

impl TransientBindGroupBufferHandle {
    pub fn create<T: PassNodeBuilderExt>(
        &self,
        pass_node_builder: &mut T,
    ) -> TransientBindGroupBuffer {
        let buffer = pass_node_builder.read(self.buffer.clone());

        TransientBindGroupBuffer {
            buffer,
            size: self.size,
            offset: self.offset,
        }
    }
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

impl TransientBindGroupHandle {
    pub fn create<T: PassNodeBuilderExt>(&self, pass_builder: &mut T) -> TransientBindGroup {
        let entries = self
            .entries
            .iter()
            .map(|handle| {
                let resource = match &handle.resource {
                    TransientBindGroupResourceHandle::Buffer(buffer) => {
                        TransientBindGroupResource::Buffer(buffer.create(pass_builder))
                    }
                    TransientBindGroupResourceHandle::Sampler(sampler) => {
                        TransientBindGroupResource::Sampler(sampler.clone())
                    }
                    TransientBindGroupResourceHandle::TextureView(texture_view) => {
                        TransientBindGroupResource::TextureView(texture_view.create(pass_builder))
                    }
                    TransientBindGroupResourceHandle::TextureViewArray(texture_view_array) => {
                        let texture_view_array = texture_view_array
                            .iter()
                            .map(|texture_view| texture_view.create(pass_builder))
                            .collect();

                        TransientBindGroupResource::TextureViewArray(texture_view_array)
                    }
                };

                TransientBindGroupEntry {
                    binding: handle.binding,
                    resource,
                }
            })
            .collect();

        TransientBindGroup {
            label: self.label.clone(),
            layout: self.layout.clone(),
            entries,
        }
    }

    pub fn build(layout: &BindGroupLayout) -> TransientBindGroupHandleBuilder {
        TransientBindGroupHandleBuilder::new(None, layout.clone())
    }
}

pub trait IntoTransientBindGroupResourceHandle {
    fn into_handle(self) -> TransientBindGroupResourceHandle;
}

impl IntoTransientBindGroupResourceHandle for TransientBindGroupBufferHandle {
    fn into_handle(self) -> TransientBindGroupResourceHandle {
        TransientBindGroupResourceHandle::Buffer(self)
    }
}

impl IntoTransientBindGroupResourceHandle for Sampler {
    fn into_handle(self) -> TransientBindGroupResourceHandle {
        TransientBindGroupResourceHandle::Sampler(self)
    }
}

impl IntoTransientBindGroupResourceHandle for TransientBindGroupTextureViewHandle {
    fn into_handle(self) -> TransientBindGroupResourceHandle {
        TransientBindGroupResourceHandle::TextureView(self)
    }
}

impl IntoTransientBindGroupResourceHandle for Vec<TransientBindGroupTextureViewHandle> {
    fn into_handle(self) -> TransientBindGroupResourceHandle {
        TransientBindGroupResourceHandle::TextureViewArray(self)
    }
}

impl<T: Clone + IntoTransientBindGroupResourceHandle> IntoTransientBindGroupResourceHandle for &T {
    fn into_handle(self) -> TransientBindGroupResourceHandle {
        self.clone().into_handle()
    }
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
