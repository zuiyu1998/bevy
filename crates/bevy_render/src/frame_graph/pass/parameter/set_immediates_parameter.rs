use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct SetImmediatesParameter {
    pub offset: u32,
    pub data: Vec<u8>,
}

impl RenderPassCommand for SetImmediatesParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.set_immediates(self.offset, &self.data);
    }
}
