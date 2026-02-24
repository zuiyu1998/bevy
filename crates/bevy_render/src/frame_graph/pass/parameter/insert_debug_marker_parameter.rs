use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct InsertDebugMarkerParameter {
    pub label: String,
}

impl RenderPassCommand for InsertDebugMarkerParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.insert_debug_marker(&self.label);
    }
}
