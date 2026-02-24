use crate::frame_graph::{
    RenderPassCommand, RenderPassContext, ResourceRead, ResourceRef, TransientBuffer,
};

pub struct MultiDrawIndexedIndirectParameter {
    pub indirect_buffer_ref: ResourceRef<TransientBuffer, ResourceRead>,
    pub indirect_offset: u64,
    pub count: u32,
}

impl RenderPassCommand for MultiDrawIndexedIndirectParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.multi_draw_indexed_indirect(
            &self.indirect_buffer_ref,
            self.indirect_offset,
            self.count,
        );
    }
}
