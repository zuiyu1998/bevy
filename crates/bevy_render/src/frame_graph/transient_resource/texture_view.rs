use wgpu::{TextureAspect, TextureFormat, TextureUsages, TextureView, TextureViewDimension};

use crate::frame_graph::{
    pass::PassContext, ResourceRead, ResourceRef, ResourceView, ResourceWrite, TransientTexture,
};

pub type TransientTextureViewRead = TransientTextureView<ResourceRead>;

pub type TransientTextureViewWrite = TransientTextureView<ResourceWrite>;

#[derive(Default, Clone, Debug, PartialEq, Eq)]
pub struct TransientTextureViewDescriptor {
    pub label: Option<String>,
    pub format: Option<TextureFormat>,
    pub dimension: Option<TextureViewDimension>,
    pub usage: Option<TextureUsages>,
    pub aspect: TextureAspect,
    pub base_mip_level: u32,
    pub mip_level_count: Option<u32>,
    pub base_array_layer: u32,
    pub array_layer_count: Option<u32>,
}

impl TransientTextureViewDescriptor {
    pub fn from_desc(desc: &wgpu::TextureViewDescriptor<'_>) -> Self {
        Self {
            label: desc.label.as_ref().map(ToString::to_string),
            format: desc.format,
            dimension: desc.dimension,
            usage: desc.usage,
            aspect: desc.aspect,
            base_mip_level: desc.base_mip_level,
            mip_level_count: desc.mip_level_count,
            base_array_layer: desc.base_array_layer,
            array_layer_count: desc.array_layer_count,
        }
    }

    pub fn get_desc<'a>(&'a self) -> wgpu::TextureViewDescriptor<'a> {
        wgpu::TextureViewDescriptor {
            label: self.label.as_deref(),
            format: self.format,
            dimension: self.dimension,
            usage: self.usage,
            aspect: self.aspect,
            base_mip_level: self.base_mip_level,
            mip_level_count: self.mip_level_count,
            base_array_layer: self.base_array_layer,
            array_layer_count: self.array_layer_count,
        }
    }
}

pub struct TransientTextureView<ViewType> {
    pub texture: ResourceRef<TransientTexture, ViewType>,
    pub desc: TransientTextureViewDescriptor,
}

impl<ViewType: ResourceView> TransientTextureView<ViewType> {
    pub fn create_texture_view(&self, context: &PassContext) -> TextureView {
        let resource = context.get_resource(&self.texture);
        resource.resource.create_view(&self.desc.get_desc())
    }
}

impl<ViewType: ResourceView> Clone for TransientTextureView<ViewType> {
    fn clone(&self) -> Self {
        Self {
            texture: self.texture.clone(),
            desc: self.desc.clone(),
        }
    }
}

#[derive(Clone)]
pub enum TextureViewEdge {
    Read(TransientTextureViewRead),
    Write(TransientTextureViewWrite),
    Owned(TextureView),
}

impl TextureViewEdge {
    pub fn create_texture_view(&self, context: &PassContext) -> TextureView {
        match self {
            TextureViewEdge::Read(desc) => desc.create_texture_view(context),
            TextureViewEdge::Write(desc) => desc.create_texture_view(context),
            TextureViewEdge::Owned(texture_view) => texture_view.clone(),
        }
    }
}
