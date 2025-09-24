use wgpu::{Operations, TextureView};

pub struct RenderPassDepthStencilAttachment {
    /// The view to use as an attachment.
    pub view: TextureView,
    /// What operations will be performed on the depth part of the attachment.
    pub depth_ops: Option<Operations<f32>>,
    /// What operations will be performed on the stencil part of the attachment.
    pub stencil_ops: Option<Operations<u32>>,
}
