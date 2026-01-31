use wgpu::RenderPipeline;

use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct SetRenderPipelineParameter {
    pub pipeline: RenderPipeline,
}

impl RenderPassCommand for SetRenderPipelineParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.set_render_pipeline(&self.pipeline);
    }
}
