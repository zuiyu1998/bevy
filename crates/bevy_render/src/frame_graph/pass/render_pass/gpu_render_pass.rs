use wgpu::{Color, Operations, RenderPassDescriptor as WgpuRenderPassDescriptor, TextureView};

pub struct RenderPassDepthStencilAttachment {
    pub view: TextureView,
    pub depth_ops: Option<Operations<f32>>,
    pub stencil_ops: Option<Operations<u32>>,
}

impl RenderPassDepthStencilAttachment {
    pub fn get_render_pass_depth_stencil_attachment(
        &self,
    ) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: self.depth_ops,
            stencil_ops: self.stencil_ops,
        }
    }
}

pub struct RenderPassColorAttachment {
    pub view: TextureView,
    pub depth_slice: Option<u32>,
    pub resolve_target: Option<TextureView>,
    pub ops: Operations<Color>,
}

impl RenderPassColorAttachment {
    fn get_wgpu_render_pass_color_attachment(&self) -> wgpu::RenderPassColorAttachment<'_> {
        wgpu::RenderPassColorAttachment {
            view: &self.view,
            depth_slice: self.depth_slice,
            resolve_target: self.resolve_target.as_ref(),
            ops: self.ops,
        }
    }
}
pub struct RenderPassDescriptor {
    pub label: Option<String>,
    pub color_attachments: Vec<Option<RenderPassColorAttachment>>,
    pub depth_stencil_attachment: Option<RenderPassDepthStencilAttachment>,
}

pub struct GpuRenderPass(wgpu::RenderPass<'static>);

impl GpuRenderPass {
    pub(crate) fn get_render_pass_mut(&mut self) -> &mut wgpu::RenderPass<'static> {
        &mut self.0
    }

    pub fn begin_render_pass(
        command_encoder: &mut wgpu::CommandEncoder,
        desc: &RenderPassDescriptor,
    ) -> Self {
        let depth_stencil_attachment =
            desc.depth_stencil_attachment
                .as_ref()
                .map(|depth_stencil_attachment| {
                    depth_stencil_attachment.get_render_pass_depth_stencil_attachment()
                });

        let color_attachments = desc
            .color_attachments
            .iter()
            .map(|color_attachment| {
                color_attachment.as_ref().map(|color_attachment| {
                    color_attachment.get_wgpu_render_pass_color_attachment()
                })
            })
            .collect::<Vec<_>>();

        let render_pass = command_encoder.begin_render_pass(&WgpuRenderPassDescriptor {
            label: desc.label.as_deref(),
            color_attachments: &color_attachments,
            depth_stencil_attachment,
            ..Default::default()
        });

        GpuRenderPass(render_pass.forget_lifetime())
    }
}
