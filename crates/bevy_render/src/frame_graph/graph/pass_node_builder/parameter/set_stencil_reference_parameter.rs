use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct SetStencilReferenceParameter {
    pub reference: u32,
}

impl RenderPassCommand for SetStencilReferenceParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.set_stencil_reference(self.reference);
    }
}
