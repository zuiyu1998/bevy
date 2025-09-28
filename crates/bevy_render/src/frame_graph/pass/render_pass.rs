use crate::{
    frame_graph::{
        PassCommand, PassContext, RenderPassColorAttachment, RenderPassDepthStencilAttachment,
        ResourceTable,
    },
    renderer::RenderDevice,
};
use wgpu::{CommandEncoder, RenderPass as WgpuRenderPass};

pub struct RenderPassDescriptor {
    pub label: Option<String>,
    pub color_attachments: Vec<Option<RenderPassColorAttachment>>,
    pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachment>,
}

pub fn create_render_pass(
    command_encoder: &mut CommandEncoder,
    desc: &RenderPassDescriptor,
) -> WgpuRenderPass<'static> {
    let render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: desc.label.as_deref(),
        color_attachments: &desc
            .color_attachments
            .iter()
            .map(|attachment| {
                attachment
                    .as_ref()
                    .map(|attachment| wgpu::RenderPassColorAttachment {
                        view: &attachment.view,
                        resolve_target: attachment.resolve_target.as_ref(),
                        ops: attachment.ops,
                        depth_slice: attachment.depth_slice,
                    })
            })
            .collect::<Vec<_>>(),
        depth_stencil_attachment: desc.depth_stencil_attachment.as_ref().map(|attachment| {
            wgpu::RenderPassDepthStencilAttachment {
                view: &attachment.view,
                depth_ops: attachment.depth_ops,
                stencil_ops: attachment.stencil_ops,
            }
        }),
        ..Default::default()
    });
    render_pass.forget_lifetime()
}

pub struct RenderPass {
    pub desc: RenderPassDescriptor,
    pub commands: Vec<Box<dyn RenderPassCommand>>,
}

impl PassCommand for RenderPass {
    fn execute(&self, ctx: &mut PassContext) {
        let command_encoder = ctx.get_or_create_command_encoder(None);

        let render_pass = create_render_pass(command_encoder, &self.desc);
        let mut ctx = RenderPassContext {
            render_pass,
            resource_table: ctx.resource_table,
            render_device: ctx.render_device.clone(),
        };

        for command in &self.commands {
            command.execute(&mut ctx);
        }
    }
}

pub struct RenderPassContext<'a> {
    pub render_pass: wgpu::RenderPass<'static>,
    pub resource_table: &'a ResourceTable,
    pub render_device: RenderDevice,
}

pub trait RenderPassCommand: 'static + Send + Sync {
    fn execute(&self, _ctx: &mut RenderPassContext);
}
