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
    command_buffer_queue: Vec<FrameGraphCommandBuffer>,
    resource_table: ResourceTable,
    pipeline_cache: &'a PipelineCache,
}

impl<'a> RenderContext<'a> {
    pub fn new(device: &'a RenderDevice, pipeline_cache: &'a PipelineCache) -> Self {
        Self {
            device,
            command_buffer_queue: vec![],
            resource_table: Default::default(),
            pipeline_cache,
        }
    }

    pub fn begin_render_pass<'b>(
        &'b mut self,
        render_pass_info: &RenderPassInfo,
    ) -> Result<TrackedRenderPass<'b, 'a>>
    where
        'a: 'b,
    {
        let mut command_buffer = self.device.create_command_buffer();

        command_buffer.begin_render_pass(&self.resource_table, render_pass_info)?;

        Ok(TrackedRenderPass {
            render_context: self,
            command_buffer,
        })
    }

    pub fn finish(self) -> Vec<FrameGraphCommandBuffer> {
        self.command_buffer_queue
    }

}

pub struct TrackedRenderPass<'a, 'b> {
    command_buffer: FrameGraphCommandBuffer,
    render_context: &'a mut RenderContext<'b>,
}

impl<'a, 'b> TrackedRenderPass<'a, 'b> {
    pub fn set_index_buffer(
        &mut self,
        buffer_ref: &ResourceRef<FrameGraphBuffer, GpuRead>,
        index_format: IndexFormat,
    ) -> Result<()> {
        self.command_buffer.set_index_buffer(
            &self.render_context.resource_table,
            buffer_ref,
            index_format,
        )
    }

    pub fn set_vertex_buffer(
        &mut self,
        slot: u32,
        buffer_ref: &ResourceRef<FrameGraphBuffer, GpuRead>,
    ) -> Result<()> {
        self.command_buffer
            .set_vertex_buffer(&self.render_context.resource_table, buffer_ref, slot)
    }

    pub fn set_bind_group(
        &mut self,
        bind_group_ref: Option<&BindGroupRef>,
        index: u32,
        offsets: &[u32],
    ) -> Result<()> {
        self.command_buffer.set_bind_group(
            &self.render_context.resource_table,
            bind_group_ref,
            index,
            offsets,
        )
    }

    pub fn set_pipeline(&mut self, id: &CachedRenderPipelineId) -> Result<()> {
        let pipeline = self
            .render_context
            .pipeline_cache
            .get_render_pipeline(*id)
            .ok_or(ErrorKind::RenderPipelineNotFound)?;

        self.command_buffer.set_pipeline(pipeline);

        Ok(())
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.command_buffer
            .draw_indexed(indices, base_vertex, instances);
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.command_buffer.draw(vertices, instances);
    }

    pub fn end_render_pass(mut self) {
        self.command_buffer.end_render_pass();

        self.render_context
            .command_buffer_queue
            .push(self.command_buffer);
    }
}
