use wgpu::{ComputePipeline, RenderPipeline};

pub enum Pipeline {
    RenderPipeline(RenderPipeline),
    ComputePipeline(ComputePipeline),
}

impl Pipeline {
    pub fn get_render_pipeline(&self) -> Option<&RenderPipeline> {
        match self {
            Pipeline::RenderPipeline(res) => Some(res),
            _ => None,
        }
    }

    pub fn get_compute_pipeline(&self) -> Option<&ComputePipeline> {
        match self {
            Pipeline::ComputePipeline(res) => Some(res),
            _ => None,
        }
    }
}

pub trait GetPipelineContainer {
    fn get_pipeline_container(&self) -> PipelineContainer;
}

pub struct PipelineContainer(Vec<Option<Pipeline>>);

impl PipelineContainer {
    pub fn get_render_pipeline(&self, id: usize) -> Option<&RenderPipeline> {
        self.0.get(id).and_then(|pipeline| {
            pipeline
                .as_ref()
                .and_then(|pipeline| pipeline.get_render_pipeline())
        })
    }

    pub fn get_compute_pipeline(&self, id: usize) -> Option<&ComputePipeline> {
        self.0.get(id).and_then(|pipeline| {
            pipeline
                .as_ref()
                .and_then(|pipeline| pipeline.get_compute_pipeline())
        })
    }
}
