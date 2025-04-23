use std::sync::Arc;

#[derive(Clone, Default)]
pub struct TextureInfo {}

pub struct FrameGraphTexture {
    pub value: wgpu::Texture,
    pub desc: TextureInfo,
}

impl FrameGraphTexture {
    pub fn new_arc(value: wgpu::Texture) -> Arc<FrameGraphTexture> {
        Arc::new(FrameGraphTexture {
            value,
            desc: TextureInfo::default(),
        })
    }
}
