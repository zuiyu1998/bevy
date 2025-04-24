use std::ops::Range;

use wgpu::{IndexFormat, Operations};

use crate::{frame_graph::*, render_resource::RenderPipeline, renderer::RenderDevice};

pub struct FrameGraphCommandBuffer {
    device: RenderDevice,
    command_encoder: Option<wgpu::CommandEncoder>,
    render_pass: Option<wgpu::RenderPass<'static>>,
    command_buffer: Option<wgpu::CommandBuffer>,
}

impl FrameGraphCommandBuffer {
    pub fn command_buffer(self) -> Option<wgpu::CommandBuffer> {
        self.command_buffer
    }

    pub fn new(device: &RenderDevice) -> Self {
        Self {
            device: device.clone(),
            command_encoder: None,
            render_pass: None,
            command_buffer: None,
        }
    }
}

impl CommandBufferTrait for FrameGraphCommandBuffer {
    fn begin_render_pass(
        &mut self,
        resource_table: &ResourceTable,
        render_pass_info: &RenderPassInfo,
    ) -> Result<()> {
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let render_pass_view = RenderPassView::prepare_view(&resource_table, &render_pass_info)?;

        let mut color_attachments = vec![];
        for color_attachment in render_pass_view.color_attachments.iter() {
            color_attachments.push(ColorAttachment::new(color_attachment));
        }

        let color_attachments = color_attachments
            .iter()
            .map(|color_attachment| Some(color_attachment.get_render_pass_color_attachment()))
            .collect::<Vec<_>>();

        let render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            ..Default::default()
        });

        let render_pass = render_pass.forget_lifetime();
        self.render_pass = Some(render_pass);

        self.command_encoder = Some(command_encoder);
        Ok(())
    }

    fn end_render_pass(&mut self) {
        if let Some(render_pass) = self.render_pass.take() {
            drop(render_pass);
        }

        if let Some(command_encoder) = self.command_encoder.take() {
            let command_buffer = command_encoder.finish();
            self.command_buffer = Some(command_buffer);
        }
    }

    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.render_pass.as_mut().unwrap().draw(vertices, instances);
    }

    fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.render_pass
            .as_mut()
            .unwrap()
            .draw_indexed(indices, base_vertex, instances);
    }

    fn set_pipeline(&mut self, pipeline: &RenderPipeline) {
        self.render_pass.as_mut().unwrap().set_pipeline(pipeline);
    }

    fn set_bind_group(
        &mut self,
        resource_table: &ResourceTable,
        bind_group_ref: Option<&BindGroupRef>,
        index: u32,
        offsets: &[u32],
    ) -> Result<()> {
        if bind_group_ref.is_none() {
            self.render_pass
                .as_mut()
                .unwrap()
                .set_bind_group(index, None, offsets);

            return Ok(());
        }

        let bind_group_ref = bind_group_ref.unwrap();

        let bind_group_view = BindGroupView::prepare_view(resource_table, bind_group_ref)?;

        let bind_group =
            self.device
                .create_bind_group(None, &bind_group_view.layout, bind_group_view.entries);

        self.render_pass
            .as_mut()
            .unwrap()
            .set_bind_group(index, &bind_group, offsets);

        Ok(())
    }

    fn set_vertex_buffer(
        &mut self,
        resource_table: &ResourceTable,
        buffer_ref: &ResourceRef<FrameGraphBuffer, GpuRead>,
        slot: u32,
    ) -> Result<()> {
        let buffer = resource_table
            .get_resource(buffer_ref)
            .ok_or(ErrorKind::ResourceNotFound)?;

        self.render_pass
            .as_mut()
            .unwrap()
            .set_vertex_buffer(slot, *buffer.value.slice(0..));

        Ok(())
    }

    fn set_index_buffer(
        &mut self,
        resource_table: &ResourceTable,
        buffer_ref: &ResourceRef<FrameGraphBuffer, GpuRead>,
        index_format: IndexFormat,
    ) -> Result<()> {
        let buffer = resource_table
            .get_resource(buffer_ref)
            .ok_or(ErrorKind::ResourceNotFound)?;

        self.render_pass
            .as_mut()
            .unwrap()
            .set_index_buffer(*buffer.value.slice(0..), index_format);

        Ok(())
    }
}

#[derive(Clone)]
pub struct RenderPassInfo {
    pub color_attachments: Vec<ColorAttachmentInfo>,
}

pub struct ColorAttachment {
    pub view: wgpu::TextureView,
    pub resolve_target: Option<wgpu::TextureView>,
    pub ops: Operations<wgpu::Color>,
}

impl ColorAttachment {
    pub fn get_render_pass_color_attachment(&self) -> wgpu::RenderPassColorAttachment {
        wgpu::RenderPassColorAttachment {
            view: &self.view,
            resolve_target: self.resolve_target.as_ref(),
            ops: self.ops.clone(),
        }
    }

    pub fn new(color_attachment: &ColorAttachmentView) -> Self {
        Self {
            view: color_attachment
                .view
                .value
                .create_view(&wgpu::TextureViewDescriptor {
                    ..Default::default()
                }),
            resolve_target: color_attachment.resolve_target.map(|texture| {
                texture.value.create_view(&wgpu::TextureViewDescriptor {
                    ..Default::default()
                })
            }),
            ops: color_attachment.ops.clone(),
        }
    }
}

pub struct RenderPassView<'a> {
    pub color_attachments: Vec<ColorAttachmentView<'a>>,
}

impl<'a> ResourceView<'a> for RenderPassView<'a> {
    type ViewRef = RenderPassInfo;
    fn prepare_view(
        resource_table: &'a ResourceTable,
        view_ref: &'a Self::ViewRef,
    ) -> Result<Self> {
        let mut color_attachments = vec![];

        for color_attachment in view_ref.color_attachments.iter() {
            color_attachments.push(ColorAttachmentView::prepare_view(
                resource_table,
                &color_attachment,
            )?);
        }

        Ok(RenderPassView { color_attachments })
    }
}

pub struct ColorAttachmentView<'a> {
    pub view: &'a FrameGraphTexture,
    pub resolve_target: Option<&'a FrameGraphTexture>,
    pub ops: Operations<wgpu::Color>,
}

#[derive(Clone)]
pub struct ColorAttachmentInfo {
    pub view: ResourceRef<FrameGraphTexture, GpuRead>,
    pub resolve_target: Option<ResourceRef<FrameGraphTexture, GpuRead>>,
    pub ops: Operations<wgpu::Color>,
}

impl<'a> ResourceView<'a> for ColorAttachmentView<'a> {
    type ViewRef = ColorAttachmentInfo;
    fn prepare_view(
        resource_table: &'a ResourceTable,
        view_ref: &'a Self::ViewRef,
    ) -> Result<Self> {
        let view = resource_table
            .get_resource(&view_ref.view)
            .ok_or(ErrorKind::ResourceNotFound)?;

        let mut resolve_target = None;

        if let Some(texture_ref) = view_ref.resolve_target.clone() {
            let target = resource_table
                .get_resource(&texture_ref)
                .ok_or(ErrorKind::ResourceNotFound)?;

            resolve_target = Some(target)
        }

        Ok(ColorAttachmentView {
            view,
            resolve_target,
            ops: view_ref.ops.clone(),
        })
    }
}

pub trait CommandBufferTrait: 'static + Sync + Send {
    fn begin_render_pass(
        &mut self,
        resource_table: &ResourceTable,
        render_pass_info: &RenderPassInfo,
    ) -> Result<()>;

    fn end_render_pass(&mut self);

    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>);

    fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>);

    fn set_pipeline(&mut self, pipeline: &RenderPipeline);

    fn set_bind_group(
        &mut self,
        resource_table: &ResourceTable,
        bind_group_ref: Option<&BindGroupRef>,
        index: u32,
        offsets: &[u32],
    ) -> Result<()>;

    fn set_vertex_buffer(
        &mut self,
        resource_table: &ResourceTable,
        buffer_ref: &ResourceRef<FrameGraphBuffer, GpuRead>,
        slot: u32,
    ) -> Result<()>;

    fn set_index_buffer(
        &mut self,
        resource_table: &ResourceTable,
        buffer_ref: &ResourceRef<FrameGraphBuffer, GpuRead>,
        index_format: IndexFormat,
    ) -> Result<()>;
}
