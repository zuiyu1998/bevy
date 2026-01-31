use crate::frame_graph::{RenderPassCommand, RenderPassContext};
use std::ops::Range;

pub struct DrawIndexedParameter {
    pub indices: Range<u32>,
    pub base_vertex: i32,
    pub instances: Range<u32>,
}

impl RenderPassCommand for DrawIndexedParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.draw_indexed(
            self.indices.clone(),
            self.base_vertex,
            self.instances.clone(),
        );
    }
}
