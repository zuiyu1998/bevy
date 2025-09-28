mod parameter;
mod render_pass;

pub use parameter::*;
pub use render_pass::*;

use crate::{frame_graph::ResourceTable, renderer::RenderDevice};
use wgpu::{CommandBuffer, CommandEncoder};

pub struct Pass {
    pub name: Option<String>,
    pub commands: Vec<Box<dyn PassCommand>>,
}

impl Pass {
    pub fn new(name: Option<String>) -> Self {
        Self {
            name,
            commands: vec![],
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    pub fn add_command(&mut self, command: Box<dyn PassCommand>) {
        self.commands.push(command);
    }

    pub fn render(
        &self,
        render_device: &RenderDevice,
        command_buffers: &mut Vec<CommandBuffer>,
        resource_table: &ResourceTable,
    ) {
        let mut ctx = PassContext::new(render_device.clone(), resource_table);
        for command in self.commands.iter() {
            command.execute(&mut ctx);
        }
        if let Some(encoder) = ctx.command_encoder {
            command_buffers.push(encoder.finish());
        }
    }
}
pub struct PassContext<'a> {
    command_encoder: Option<CommandEncoder>,
    render_device: RenderDevice,
    resource_table: &'a ResourceTable,
}

impl<'a> PassContext<'a> {
    pub fn new(render_device: RenderDevice, resource_table: &'a ResourceTable) -> Self {
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

pub trait PassCommand: 'static + Send + Sync {
    fn execute(&self, _ctx: &mut PassContext);
}
