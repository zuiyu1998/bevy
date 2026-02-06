use crate::frame_graph::{
    RenderPassCommand, RenderPassContext, ResourceRead, ResourceRef, TransientBuffer,
};

pub struct DrawIndirectParameter {
    pub indirect_buffer_ref: ResourceRef<TransientBuffer, ResourceRead>,
    pub indirect_offset: u64,
}

impl RenderPassCommand for DrawIndirectParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.draw_indirect(&self.indirect_buffer_ref, self.indirect_offset);
    }
}
