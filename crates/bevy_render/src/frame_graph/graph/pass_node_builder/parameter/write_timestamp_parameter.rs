use wgpu::QuerySet;

use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct WriteTimestampParameter {
    pub query_set: QuerySet,
    pub query_index: u32,
}

impl RenderPassCommand for WriteTimestampParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.write_timestamp(&self.query_set, self.query_index);
    }
}
