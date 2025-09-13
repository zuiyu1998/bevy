use wgpu::CommandEncoder;

use crate::renderer::RenderDevice;

pub struct RenderContext {
    render_device: RenderDevice,
}

pub struct PassContext {
    pub enconder: CommandEncoder,
}

impl PassContext {
    pub fn new(enconder: CommandEncoder) -> Self {
        Self { enconder }
    }
}

pub trait PassCommand {
    fn execute(&self, ctx: &mut PassContext);
}

pub struct Pass {
    pub label: Option<String>,
    commands: Vec<Box<dyn PassCommand>>,
}

impl Default for Pass {
    fn default() -> Self {
        Self::new()
    }
}

impl Pass {
    pub fn new() -> Self {
        Pass {
            label: None,
            commands: vec![],
        }
    }

    pub fn render(&self, ctx: &mut RenderContext) {
        let encoder =
            ctx.render_device
                .create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor {
                    label: self.label.as_deref(),
                });

        let mut pass_ctx = PassContext::new(encoder);

        for command in self.commands.iter() {
            command.execute(&mut pass_ctx);
        }
    }
}
