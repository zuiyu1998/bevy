use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct PopDebugGroupParameter;

impl RenderPassCommand for PopDebugGroupParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.pop_debug_group();
    }
}
