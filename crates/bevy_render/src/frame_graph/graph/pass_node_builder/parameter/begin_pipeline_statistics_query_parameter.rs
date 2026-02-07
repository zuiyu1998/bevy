use wgpu::QuerySet;

use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct BeginPipelineStatisticsQueryParameter {
    pub query_set: QuerySet,
    pub query_index: u32,
}

impl RenderPassCommand for BeginPipelineStatisticsQueryParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.begin_pipeline_statistics_query(&self.query_set, self.query_index);
    }
}
