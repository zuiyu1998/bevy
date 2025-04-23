use std::ops::Range;

use wgpu::IndexFormat;

use crate::{
    render_resource::{CachedRenderPipelineId, PipelineCache},
    renderer::RenderDevice,
};

use super::{
    BindGroupRef, CommandBufferTrait, DeviceTrait, ErrorKind, FrameGraphBuffer,
    FrameGraphCommandBuffer, GpuRead, RenderPassInfo, ResourceRef, ResourceTable, Result,
};

pub struct RenderContext<'a> {
    device: &'a RenderDevice,
    command_buffer: Option<FrameGraphCommandBuffer>,
    command_buffer_queue: Vec<FrameGraphCommandBuffer>,
    resource_table: ResourceTable,
    pipeline_cache: &'a PipelineCache,
}

impl<'a> RenderContext<'a> {
    pub fn new(device: &'a RenderDevice, pipeline_cache: &'a PipelineCache) -> Self {
        Self {
            device,
            command_buffer: None,
            command_buffer_queue: vec![],
            resource_table: Default::default(),
            pipeline_cache,
        }
    }

    pub fn set_index_buffer(
        &mut self,
        buffer_ref: &ResourceRef<FrameGraphBuffer, GpuRead>,
        index_format: IndexFormat,
    ) -> Result<()> {
        self.command_buffer.as_mut().unwrap().set_index_buffer(
            &self.resource_table,
            buffer_ref,
            index_format,
        )
    }

    pub fn set_vertex_buffer(
        &mut self,
        slot: u32,
        buffer_ref: &ResourceRef<FrameGraphBuffer, GpuRead>,
    ) -> Result<()> {
        self.command_buffer.as_mut().unwrap().set_vertex_buffer(
            &self.resource_table,
            buffer_ref,
            slot,
        )
    }

    pub fn set_bind_group(
        &mut self,
        bind_group_ref: Option<&BindGroupRef>,
        index: u32,
        offsets: &[u32],
    ) -> Result<()> {
        self.command_buffer.as_mut().unwrap().set_bind_group(
            &self.resource_table,
            bind_group_ref,
            index,
            offsets,
        )
    }

    pub fn set_pipeline(&mut self, id: &CachedRenderPipelineId) -> Result<()> {
        let pipeline = self
            .pipeline_cache
            .get_render_pipeline(*id)
            .ok_or(ErrorKind::RenderPipelineNotFound)?;

        self.command_buffer.as_mut().unwrap().set_pipeline(pipeline);

        Ok(())
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.command_buffer
            .as_mut()
            .unwrap()
            .draw_indexed(indices, base_vertex, instances);
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.command_buffer
            .as_mut()
            .unwrap()
            .draw(vertices, instances);
    }

    pub fn begin_render_pass(&mut self, render_pass_info: &RenderPassInfo) -> Result<()> {
        self.flush();

        let mut command_buffer = self.device.create_command_buffer();

        command_buffer.begin_render_pass(&self.resource_table, render_pass_info)?;

        self.command_buffer = Some(command_buffer);

        Ok(())
    }

    pub fn flush(&mut self) {
        if let Some(mut command_buffer) = self.command_buffer.take() {
            command_buffer.end_render_pass();

            self.command_buffer_queue.push(command_buffer);
        }
    }
}
