mod bind_group;
mod buffer;
mod cache;
mod texture;
mod texture_view;

pub use bind_group::*;
pub use buffer::*;
pub use cache::*;
pub use texture::*;
pub use texture_view::*;

use alloc::sync::Arc;
use bevy_render::renderer::RenderDevice;
use core::fmt::Debug;
use wgpu::{BindGroup, BindGroupEntry};

pub trait TransientResourceCreator {
    fn create_transient_resource(
        &self,
        desc: &AnyTransientResourceDescriptor,
    ) -> AnyTransientResource;
    fn create_transient_bind_group(&self, desc: &TransientBindGroupDescriptor) -> BindGroup;
}

impl TransientResourceCreator for RenderDevice {
    fn create_transient_bind_group(&self, desc: &TransientBindGroupDescriptor) -> BindGroup {
        let entries = desc
            .entries
            .iter()
            .map(|entry| match entry.resource {
                GpuBindingResource::Buffer(ref binding) => (
                    entry.binding,
                    TransientBindingResource::Buffer(binding.get_binding()),
                ),
                GpuBindingResource::BufferArray(ref bindings) => (
                    entry.binding,
                    TransientBindingResource::BufferArray(
                        bindings
                            .iter()
                            .map(|binding| binding.get_binding())
                            .collect(),
                    ),
                ),
                GpuBindingResource::Sampler(ref binding) => {
                    (entry.binding, TransientBindingResource::Sampler(binding))
                }
                GpuBindingResource::SamplerArray(ref bindings) => (
                    entry.binding,
                    TransientBindingResource::SamplerArray(bindings.iter().collect()),
                ),
                GpuBindingResource::TextureView(ref binding) => (
                    entry.binding,
                    TransientBindingResource::TextureView(binding),
                ),
                GpuBindingResource::TextureViewArray(ref bindings) => (
                    entry.binding,
                    TransientBindingResource::TextureViewArray(bindings.iter().collect()),
                ),
            })
            .collect::<Vec<_>>();

        self.wgpu_device()
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: desc.label.as_deref(),
                layout: &desc.layout,
                entries: &entries
                    .iter()
                    .map(|(binding, resource)| BindGroupEntry {
                        binding: *binding,
                        resource: resource.get_binding_resource(),
                    })
                    .collect::<Vec<_>>(),
            })
    }

    fn create_transient_resource(
        &self,
        desc: &AnyTransientResourceDescriptor,
    ) -> AnyTransientResource {
        match desc {
            AnyTransientResourceDescriptor::Texture(desc) => {
                let resource = self.wgpu_device().create_texture(&desc.get_desc());
                TransientTexture {
                    resource,
                    desc: desc.clone(),
                }
                .into()
            }
            AnyTransientResourceDescriptor::Buffer(desc) => {
                let resource = self.wgpu_device().create_buffer(&desc.get_desc());
                TransientBuffer {
                    resource,
                    desc: desc.clone(),
                }
                .into()
            }
        }
    }
}

#[derive(Clone)]
pub enum VirtualResource {
    Setuped(AnyTransientResourceDescriptor),
    Imported(AnyArcTransientResource),
}

impl VirtualResource {
    pub fn get_desc<ResourceType: TransientResource>(&self) -> ResourceType::Descriptor {
        let desc = match self {
            VirtualResource::Imported(resource) => resource.get_desc(),
            VirtualResource::Setuped(desc) => desc.clone(),
        };

        <ResourceType::Descriptor as TransientResourceDescriptor>::borrow_resource_descriptor(&desc)
            .clone()
    }
}

#[derive(Clone)]
pub enum AnyArcTransientResource {
    Buffer(Arc<TransientBuffer>),
    Texture(Arc<TransientTexture>),
}

impl AnyArcTransientResource {
    pub fn get_desc(&self) -> AnyTransientResourceDescriptor {
        match self {
            AnyArcTransientResource::Buffer(res) => {
                AnyTransientResourceDescriptor::Buffer(res.desc.clone())
            }
            AnyArcTransientResource::Texture(res) => {
                AnyTransientResourceDescriptor::Texture(res.desc.clone())
            }
        }
    }
}

pub trait IntoAnyArcTransientResource: TransientResource {
    fn into_arc_transient_resource(self: Arc<Self>) -> AnyArcTransientResource;
}

pub enum AnyTransientResource {
    OwnedBuffer(TransientBuffer),
    ImportedBuffer(Arc<TransientBuffer>),
    OwnedTexture(TransientTexture),
    ImportedTexture(Arc<TransientTexture>),
}

impl From<TransientBuffer> for AnyTransientResource {
    fn from(value: TransientBuffer) -> Self {
        AnyTransientResource::OwnedBuffer(value)
    }
}

impl From<Arc<TransientBuffer>> for AnyTransientResource {
    fn from(value: Arc<TransientBuffer>) -> Self {
        AnyTransientResource::ImportedBuffer(value)
    }
}

impl From<TransientTexture> for AnyTransientResource {
    fn from(value: TransientTexture) -> Self {
        AnyTransientResource::OwnedTexture(value)
    }
}

impl From<Arc<TransientTexture>> for AnyTransientResource {
    fn from(value: Arc<TransientTexture>) -> Self {
        AnyTransientResource::ImportedTexture(value)
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum AnyTransientResourceDescriptor {
    Buffer(TransientBufferDescriptor),
    Texture(TransientTextureDescriptor),
}

pub trait TransientResource: 'static {
    type Descriptor: TransientResourceDescriptor;

    fn borrow_resource(res: &AnyTransientResource) -> &Self;

    fn get_desc(&self) -> &Self::Descriptor;
}

pub trait TransientResourceDescriptor:
    'static + Clone + Debug + Into<AnyTransientResourceDescriptor> + PartialEq
{
    type Resource: TransientResource;

    fn borrow_resource_descriptor(res: &AnyTransientResourceDescriptor) -> &Self;
}

pub trait TypeEquals {
    type Other;
    fn same(value: Self) -> Self::Other;
}

impl<T: Sized> TypeEquals for T {
    type Other = Self;
    fn same(value: Self) -> Self::Other {
        value
    }
}
