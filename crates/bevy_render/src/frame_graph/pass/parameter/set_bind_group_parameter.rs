use crate::frame_graph::{RenderPassCommand, RenderPassContext, TransientBindGroup};

pub struct SetBindGroupParameter {
    pub index: u32,
    pub bind_group: TransientBindGroup,
    pub offsets: Vec<u32>,
}

impl RenderPassCommand for SetBindGroupParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.set_bind_group(self.index, &self.bind_group, &self.offsets);
    }
}
