use crate::frame_graph::{
    RenderPassCommand, RenderPassContext, ResourceRead, ResourceRef, TransientBuffer,
};

pub struct DrawIndexedIndirectParameter {
    pub indirect_buffer_ref: ResourceRef<TransientBuffer, ResourceRead>,
    pub indirect_offset: u64,
}

impl RenderPassCommand for DrawIndexedIndirectParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.draw_indexed_indirect(&self.indirect_buffer_ref, self.indirect_offset);
    }
}
