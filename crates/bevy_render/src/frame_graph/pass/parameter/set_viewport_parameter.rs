use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct SetViewportParameter {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

impl RenderPassCommand for SetViewportParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.set_viewport(
            self.x,
            self.y,
            self.width,
            self.height,
            self.min_depth,
            self.max_depth,
        );
    }
}
