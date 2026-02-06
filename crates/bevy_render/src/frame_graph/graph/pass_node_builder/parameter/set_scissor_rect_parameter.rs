use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct SetScissorRectParameter {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl RenderPassCommand for SetScissorRectParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.set_scissor_rect(self.x, self.y, self.width, self.height);
    }
}
