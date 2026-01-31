use std::num::NonZero;

use wgpu::{
    BindGroup, BindGroupLayout, BindingResource, Buffer, BufferAddress, BufferSize, Sampler,
    TextureView,
};

use crate::frame_graph::{
    PassContext, ResourceRead, ResourceRef, TransientBuffer, TransientResourceCreator,
    TransientTexture, TransientTextureViewDescriptor,
};

#[derive(Clone, PartialEq, Eq)]
pub struct TransientBindGroupBuffer {
    pub buffer: ResourceRef<TransientBuffer, ResourceRead>,
    pub size: Option<NonZero<u64>>,
    pub offset: u64,
}

#[derive(Clone, PartialEq, Eq)]
pub struct TransientBindGroupTextureView {
    pub texture: ResourceRef<TransientTexture, ResourceRead>,
    pub texture_view_desc: TransientTextureViewDescriptor,
}

#[derive(Clone, PartialEq, Eq)]
pub enum TransientBindGroupResource {
    Buffer(TransientBindGroupBuffer),
    Sampler(Sampler),
    TextureView(TransientBindGroupTextureView),
    TextureViewArray(Vec<TransientBindGroupTextureView>),
}

#[derive(Clone, PartialEq, Eq)]
pub struct TransientBindGroupEntry {
    pub binding: u32,
    pub resource: TransientBindGroupResource,
}

#[derive(Clone)]
pub struct GpuBindGroupEntry {
    pub binding: u32,
    pub resource: GpuBindingResource,
}

#[derive(Clone, Debug)]
pub struct BufferBinding {
    pub buffer: Buffer,
    pub offset: BufferAddress,
    pub size: Option<BufferSize>,
}

impl BufferBinding {
    pub(crate) fn get_binding<'a>(&'a self) -> wgpu::BufferBinding<'a> {
        wgpu::BufferBinding {
            buffer: &self.buffer,
            size: self.size,
            offset: self.offset,
        }
    }
}

#[derive(Clone)]
pub enum GpuBindingResource {
    Buffer(BufferBinding),
    BufferArray(Vec<BufferBinding>),
    Sampler(Sampler),
    SamplerArray(Vec<Sampler>),
    TextureView(TextureView),
    TextureViewArray(Vec<TextureView>),
}

impl TransientBindGroupEntry {
    pub fn get_gpu_bind_group_entry(&self, context: &PassContext<'_>) -> GpuBindGroupEntry {
        match &self.resource {
            TransientBindGroupResource::Buffer(binding) => {
                let buffer = context.resource_table().get_resource(&binding.buffer);

                GpuBindGroupEntry {
                    binding: self.binding,
                    resource: GpuBindingResource::Buffer(BufferBinding {
                        buffer: buffer.resource.clone(),
                        offset: binding.offset,
                        size: binding.size,
                    }),
                }
            }
            TransientBindGroupResource::Sampler(sampler) => GpuBindGroupEntry {
                binding: self.binding,
                resource: GpuBindingResource::Sampler(sampler.clone()),
            },
            TransientBindGroupResource::TextureView(binding) => {
                let texture = context.resource_table().get_resource(&binding.texture);
                let texture_view = texture
                    .resource
                    .create_view(&binding.texture_view_desc.get_desc());

                GpuBindGroupEntry {
                    binding: self.binding,
                    resource: GpuBindingResource::TextureView(texture_view),
                }
            }
            TransientBindGroupResource::TextureViewArray(bindings) => {
                let bindings = bindings
                    .iter()
                    .map(|binding| {
                        let texture = context.resource_table().get_resource(&binding.texture);
                        texture
                            .resource
                            .create_view(&binding.texture_view_desc.get_desc())
                    })
                    .collect();

                GpuBindGroupEntry {
                    binding: self.binding,
                    resource: GpuBindingResource::TextureViewArray(bindings),
                }
            }
        }
    }
}

pub enum TransientBindingResource<'a> {
    Buffer(wgpu::BufferBinding<'a>),
    BufferArray(Vec<wgpu::BufferBinding<'a>>),
    Sampler(&'a Sampler),
    SamplerArray(Vec<&'a Sampler>),
    TextureView(&'a TextureView),
    TextureViewArray(Vec<&'a TextureView>),
}

impl<'a> TransientBindingResource<'a> {
    pub fn get_binding_resource(&'a self) -> BindingResource<'a> {
        match &self {
            TransientBindingResource::Buffer(v) => BindingResource::Buffer(v.clone()),
            TransientBindingResource::BufferArray(v) => BindingResource::BufferArray(v),
            TransientBindingResource::Sampler(v) => BindingResource::Sampler(v),
            TransientBindingResource::SamplerArray(v) => BindingResource::SamplerArray(v),
            TransientBindingResource::TextureView(v) => BindingResource::TextureView(v),
            TransientBindingResource::TextureViewArray(v) => BindingResource::TextureViewArray(v),
        }
    }
}

#[derive(Clone)]
pub struct TransientBindGroupDescriptor {
    pub label: Option<String>,
    pub layout: BindGroupLayout,
    pub entries: Vec<GpuBindGroupEntry>,
}

#[derive(Clone, PartialEq)]
pub struct TransientBindGroup {
    pub label: Option<String>,
    pub layout: BindGroupLayout,
    pub entries: Vec<TransientBindGroupEntry>,
}

impl TransientBindGroup {
    pub fn create_bind_group(&self, context: &PassContext<'_>) -> BindGroup {
        let entries = self
            .entries
            .iter()
            .map(|entry| entry.get_gpu_bind_group_entry(context))
            .collect::<Vec<_>>();

        let desc = TransientBindGroupDescriptor {
            label: self.label.clone(),
            layout: self.layout.clone(),
            entries,
        };

        context.device().create_transient_bind_group(&desc)
    }
}
