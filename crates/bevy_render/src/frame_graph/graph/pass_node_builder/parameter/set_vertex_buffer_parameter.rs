use crate::frame_graph::{
    RenderPassCommand, RenderPassContext, ResourceRead, ResourceRef, TransientBuffer,
};

pub struct SetVertexBufferParameter {
    pub slot: u32,
    pub buffer_ref: ResourceRef<TransientBuffer, ResourceRead>,
    pub offset: u64,
    pub size: u64,
}

impl RenderPassCommand for SetVertexBufferParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.set_vertex_buffer(self.slot, &self.buffer_ref, self.offset, self.size);
    }
}
