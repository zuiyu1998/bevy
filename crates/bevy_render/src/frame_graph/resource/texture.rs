use super::{
    AnyTransientResource, AnyTransientResourceDescriptor, ArcTransientResource,
    IntoArcTransientResource, TransientResource, TransientResourceDescriptor,
};
use alloc::sync::Arc;
use wgpu::{Extent3d, Texture, TextureDimension, TextureFormat, TextureUsages};

impl IntoArcTransientResource for TransientTexture {
    fn into_arc_transient_resource(self: Arc<Self>) -> ArcTransientResource {
        ArcTransientResource::Texture(self)
    }
}

pub struct TransientTexture {
    pub resource: Texture,
    pub desc: TextureDescriptor,
}

impl TransientResource for TransientTexture {
    type Descriptor = TextureDescriptor;

    fn borrow_resource(res: &AnyTransientResource) -> &Self {
        match res {
            AnyTransientResource::OwnedTexture(res) => res,
            AnyTransientResource::ImportedTexture(res) => res,
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
pub struct TextureDescriptor {
    pub label: Option<String>,
    pub size: Extent3d,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub usage: TextureUsages,
    pub view_formats: Vec<TextureFormat>,
}

impl TextureDescriptor {
    pub fn from_texture_desc(desc: &wgpu::TextureDescriptor) -> Self {
        TextureDescriptor {
            label: desc.label.map(ToString::to_string),
            size: desc.size,
            mip_level_count: desc.mip_level_count,
            sample_count: desc.sample_count,
            dimension: desc.dimension,
            format: desc.format,
            usage: desc.usage,
            view_formats: desc.view_formats.to_vec(),
        }
    }

    pub fn get_desc(&self) -> wgpu::TextureDescriptor<'_> {
        wgpu::TextureDescriptor {
            label: self.label.as_deref(),
            size: self.size,
            mip_level_count: self.mip_level_count,
            sample_count: self.sample_count,
            dimension: self.dimension,
            format: self.format,
            usage: self.usage,
            view_formats: &self.view_formats,
        }
    }
}

impl From<TextureDescriptor> for AnyTransientResourceDescriptor {
    fn from(value: TextureDescriptor) -> Self {
        AnyTransientResourceDescriptor::Texture(value)
    }
}

impl TransientResourceDescriptor for TextureDescriptor {
    type Resource = TransientTexture;

    fn borrow_resource_descriptor(res: &AnyTransientResourceDescriptor) -> &Self {
        match res {
            AnyTransientResourceDescriptor::Texture(res) => res,
            _ => {
                unimplemented!()
            }
        }
    }
}
