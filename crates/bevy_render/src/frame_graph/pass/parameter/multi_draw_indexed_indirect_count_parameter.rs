use crate::frame_graph::{
    RenderPassCommand, RenderPassContext, ResourceRead, ResourceRef, TransientBuffer,
};

pub struct MultiDrawIndexedIndirectCountParameter {
    pub indirect_buffer_ref: ResourceRef<TransientBuffer, ResourceRead>,
    pub indirect_offset: u64,
    pub count_buffer_ref: ResourceRef<TransientBuffer, ResourceRead>,
    pub count_offset: u64,
    pub max_count: u32,
}

impl RenderPassCommand for MultiDrawIndexedIndirectCountParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.multi_draw_indexed_indirect_count(
            &self.indirect_buffer_ref,
            self.indirect_offset,
            &self.count_buffer_ref,
            self.count_offset,
            self.max_count,
        );
    }
}
