mod render_pass;

pub use render_pass::*;

use wgpu::{CommandBuffer, CommandEncoder, CommandEncoderDescriptor, RenderPipeline};

use crate::{
    frame_graph::{PipelineContainer, ResourceRef, ResourceTable, ResourceView, TransientResource},
    renderer::RenderDevice,
};

pub struct PassContext<'a> {
    render_device: &'a RenderDevice,
    command_encoder: CommandEncoder,
    resource_table: &'a ResourceTable,
    pipeline_container: &'a PipelineContainer,
}

impl PassContext<'_> {
    pub fn resource_table(&self) -> &ResourceTable {
        self.resource_table
    }

    pub fn device(&self) -> &RenderDevice {
        self.render_device
    }

    pub fn get_render_pipeline(&self, id: usize) -> &RenderPipeline {
        self.pipeline_container
            .get_render_pipeline(id)
            .expect("render pipeline mut have")
    }

    pub fn finish(self) -> CommandBuffer {
        self.command_encoder.finish()
    }

    pub fn get_resource<ResourceType: TransientResource, ViewType: ResourceView>(
        &self,
        resource_ref: &ResourceRef<ResourceType, ViewType>,
    ) -> &ResourceType {
        self.resource_table.get_resource(resource_ref)
    }
}

pub trait PassCommand: 'static + Send + Sync {
    fn execute(&self, context: &mut PassContext);
}

#[derive(Default)]
pub struct Pass {
    pub label: Option<String>,
    commands: Vec<Box<dyn PassCommand>>,
}

impl Pass {
    pub fn push<T: PassCommand>(&mut self, value: T) {
        self.commands.push(Box::new(value));
    }

    pub fn render(
        &self,
        command_buffers: &mut Vec<CommandBuffer>,
        render_device: &RenderDevice,
        resource_table: &ResourceTable,
        pipeline_container: &PipelineContainer,
    ) {
        let command_encoder =
            render_device
                .wgpu_device()
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: self.label.as_deref(),
                });

        let mut pass_context = PassContext {
            render_device,
            command_encoder,
            resource_table,
            pipeline_container,
        };

        for command in self.commands.iter() {
            command.execute(&mut pass_context);
        }
        command_buffers.push(pass_context.finish());
    }
}
