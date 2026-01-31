mod context;
mod gpu_render_pass;

pub use context::*;
pub use gpu_render_pass::*;

use wgpu::{Color, Operations};

use crate::frame_graph::{PassCommand, PassContext, TextureViewEdge};

pub struct TransientRenderPassColorAttachment {
    pub view: TextureViewEdge,
    pub depth_slice: Option<u32>,
    pub resolve_target: Option<TextureViewEdge>,
    pub ops: Operations<Color>,
}

impl TransientRenderPassColorAttachment {
    pub fn create_render_pass_color_attachment(
        &self,
        context: &PassContext,
    ) -> RenderPassColorAttachment {
        RenderPassColorAttachment {
            view: self.view.create_texture_view(context),
            depth_slice: self.depth_slice,
            resolve_target: self
                .resolve_target
                .as_ref()
                .map(|resolve_target| resolve_target.create_texture_view(context)),
            ops: self.ops,
        }
    }
}

pub struct TransientRenderPassDepthStencilAttachment {
    pub view: TextureViewEdge,
    pub depth_ops: Option<Operations<f32>>,
    pub stencil_ops: Option<Operations<u32>>,
}

impl TransientRenderPassDepthStencilAttachment {
    pub fn create_render_pass_depth_stencil_attachment(
        &self,
        context: &PassContext,
    ) -> RenderPassDepthStencilAttachment {
        RenderPassDepthStencilAttachment {
            view: self.view.create_texture_view(context),
            depth_ops: self.depth_ops,
            stencil_ops: self.stencil_ops,
        }
    }
}

#[derive(Default)]
pub struct TransientRenderPassDescriptor {
    pub label: Option<String>,
    pub color_attachments: Vec<Option<TransientRenderPassColorAttachment>>,
    pub depth_stencil_attachment: Option<TransientRenderPassDepthStencilAttachment>,
}

impl TransientRenderPassDescriptor {
    pub fn create_render_pass_descriptor(&self, context: &PassContext) -> RenderPassDescriptor {
        RenderPassDescriptor {
            label: self.label.clone(),
            color_attachments: self
                .color_attachments
                .iter()
                .map(|color_attachment| {
                    color_attachment.as_ref().map(|color_attachment| {
                        color_attachment.create_render_pass_color_attachment(context)
                    })
                })
                .collect(),
            depth_stencil_attachment: self.depth_stencil_attachment.as_ref().map(
                |depth_stencil_attachment| {
                    depth_stencil_attachment.create_render_pass_depth_stencil_attachment(context)
                },
            ),
        }
    }
}

pub trait RenderPassCommand: Sync + Send + 'static {
    fn execute(&self, render_pass_context: &mut RenderPassContext);
}

#[derive(Default)]
pub struct RenderPass {
    desc: TransientRenderPassDescriptor,
    pub(crate) commands: Vec<Box<dyn RenderPassCommand>>,
}

impl RenderPass {
    pub fn set_pass_name(&mut self, name: &str) {
        self.desc.label = Some(name.to_string());
    }

    pub fn add_color_attachment(
        &mut self,
        color_attachment: Option<TransientRenderPassColorAttachment>,
    ) {
        self.desc.color_attachments.push(color_attachment);
    }
}

impl PassCommand for RenderPass {
    fn execute(&self, context: &mut PassContext) {
        let desc = self.desc.create_render_pass_descriptor(context);
        let render_pass = GpuRenderPass::begin_render_pass(&mut context.command_encoder, &desc);
        let mut render_pass_context = RenderPassContext::new(render_pass, context);

        for command in self.commands.iter() {
            command.execute(&mut render_pass_context);
        }
    }
}
