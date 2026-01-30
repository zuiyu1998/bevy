mod buffer;
mod cache;
mod texture;

pub use buffer::*;
pub use cache::*;
pub use texture::*;

use alloc::sync::Arc;
use core::fmt::Debug;

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
    'static + Clone + Debug + Into<AnyTransientResourceDescriptor>
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
