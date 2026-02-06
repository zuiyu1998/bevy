use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct PushDebugGroupParameter {
    pub label: String,
}

impl RenderPassCommand for PushDebugGroupParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.push_debug_group(&self.label);
    }
}
