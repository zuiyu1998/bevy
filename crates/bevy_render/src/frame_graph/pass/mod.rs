mod parameter;
mod render_pass;

pub use parameter::*;
pub use render_pass::*;

use crate::{frame_graph::ResourceTable, renderer::RenderDevice};
use wgpu::CommandEncoder;

pub struct Pass {
    pub name: Option<String>,
    pub commands: Vec<Box<dyn PassCommand>>,
}

pub struct PassContext {
    command_encoder: Option<CommandEncoder>,
    render_device: RenderDevice,
    resource_table: ResourceTable,
}

impl PassContext {
    pub fn new(render_device: RenderDevice, resource_table: ResourceTable) -> Self {
        Self {
            command_encoder: None,
            render_device,
            resource_table,
        }
    }

    pub fn get_or_create_command_encoder(&mut self, label: Option<&str>) -> &mut CommandEncoder {
        self.command_encoder.get_or_insert_with(|| {
            self.render_device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label })
        })
    }
}

pub trait PassCommand {
    fn execute(&self, _ctx: &mut PassContext);
}
