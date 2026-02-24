use wgpu::Color;

use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct SetBlendConstantParameter {
    pub color: Color,
}

impl RenderPassCommand for SetBlendConstantParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.set_blend_constant(&self.color);
    }
}
