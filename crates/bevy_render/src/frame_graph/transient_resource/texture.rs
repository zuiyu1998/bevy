use super::{
    AnyArcTransientResource, AnyTransientResource, AnyTransientResourceDescriptor,
    IntoAnyArcTransientResource, TransientResource, TransientResourceDescriptor,
};
use alloc::sync::Arc;
use wgpu::{Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

impl IntoAnyArcTransientResource for TransientTexture {
    fn into_arc_transient_resource(self: Arc<Self>) -> AnyArcTransientResource {
        AnyArcTransientResource::Texture(self)
    }
}

pub struct TransientTexture {
    pub resource: Texture,
    pub desc: TransientTextureDescriptor,
}

impl TransientResource for TransientTexture {
    type Descriptor = TransientTextureDescriptor;

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
pub struct TransientTextureDescriptor {
    pub label: Option<String>,
    pub size: Extent3d,
    pub mip_level_count: u32,
    pub sample_count: u32,
    pub dimension: TextureDimension,
    pub format: TextureFormat,
    pub usage: TextureUsages,
    pub view_formats: Vec<TextureFormat>,
}

//todo delete
impl Default for TransientTextureDescriptor {
    fn default() -> Self {
        TransientTextureDescriptor {
            label: None,
            size: Extent3d::default(),
            mip_level_count: Default::default(),
            sample_count: Default::default(),
            dimension: TextureDimension::D1,
            format: TextureFormat::Bc1RgbaUnorm,
            usage: TextureUsages::all(),
            view_formats: Default::default(),
        }
    }
}

impl TransientTextureDescriptor {
    pub fn from_desc(desc: &TextureDescriptor) -> Self {
        TransientTextureDescriptor {
            label: desc.label.as_ref().map(ToString::to_string),
            size: desc.size,
            mip_level_count: desc.mip_level_count,
            sample_count: desc.sample_count,
            dimension: desc.dimension,
            format: desc.format,
            usage: desc.usage,
            view_formats: desc.view_formats.to_vec(),
        }
    }

    pub fn get_desc(&self) -> TextureDescriptor<'_> {
        TextureDescriptor {
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

impl From<TransientTextureDescriptor> for AnyTransientResourceDescriptor {
    fn from(value: TransientTextureDescriptor) -> Self {
        AnyTransientResourceDescriptor::Texture(value)
    }
}

impl TransientResourceDescriptor for TransientTextureDescriptor {
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
