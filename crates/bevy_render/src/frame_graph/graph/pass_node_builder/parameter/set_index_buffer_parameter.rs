use wgpu::IndexFormat;

use crate::frame_graph::{
    RenderPassCommand, RenderPassContext, ResourceRead, ResourceRef, TransientBuffer,
};

pub struct SetIndexBufferParameter {
    pub buffer_ref: ResourceRef<TransientBuffer, ResourceRead>,
    pub index_format: IndexFormat,
    pub offset: u64,
    pub size: u64,
}

impl RenderPassCommand for SetIndexBufferParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.set_index_buffer(
            &self.buffer_ref,
            self.index_format,
            self.offset,
            self.size,
        );
    }
}
