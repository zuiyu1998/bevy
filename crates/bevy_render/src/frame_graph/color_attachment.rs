use wgpu::{Color, Operations, TextureView};

pub struct RenderPassColorAttachment {
    /// The view to use as an attachment.
    pub view: TextureView,
    /// The depth slice index of a 3D view. It must not be provided if the view is not 3D.
    pub depth_slice: Option<u32>,
    /// The view that will receive the resolved output if multisampling is used.
    ///
    /// If set, it is always written to, regardless of how [`Self::ops`] is configured.
    pub resolve_target: Option<TextureView>,
    /// What operations will be performed on this color attachment.
    pub ops: Operations<Color>,
}
