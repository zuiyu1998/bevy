use crate::frame_graph::{RenderPassCommand, RenderPassContext};
use std::ops::Range;

pub struct DrawParameter {
    pub vertices: Range<u32>,
    pub instances: Range<u32>,
}

impl RenderPassCommand for DrawParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.draw(self.vertices.clone(), self.instances.clone());
    }
}
