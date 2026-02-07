use crate::frame_graph::{RenderPassCommand, RenderPassContext};

pub struct EndPipelineStatisticsQueryParameter;

impl RenderPassCommand for EndPipelineStatisticsQueryParameter {
    fn execute(&self, render_pass_context: &mut RenderPassContext) {
        render_pass_context.end_pipeline_statistics_query();
    }
}
