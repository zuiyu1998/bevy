use core::ops::Range;

use wgpu::{Color, IndexFormat, QuerySet, RenderPipeline};

use crate::frame_graph::{
    GpuRenderPass, PassContext, ResourceRead, ResourceRef, TransientBindGroup, TransientBuffer,
};

pub struct RenderPassContext<'a, 'b> {
    render_pass: GpuRenderPass,
    pass_context: &'b mut PassContext<'a>,
}

impl<'a, 'b> RenderPassContext<'a, 'b> {
    pub fn new(render_pass: GpuRenderPass, pass_context: &'b mut PassContext<'a>) -> Self {
        RenderPassContext {
            render_pass,
            pass_context,
        }
    }

    pub fn end_pipeline_statistics_query(&mut self) {
        self.render_pass
            .get_render_pass_mut()
            .end_pipeline_statistics_query();
    }

    pub fn begin_pipeline_statistics_query(&mut self, query_set: &QuerySet, query_index: u32) {
        self.render_pass
            .get_render_pass_mut()
            .begin_pipeline_statistics_query(query_set, query_index);
    }

    pub fn write_timestamp(&mut self, query_set: &QuerySet, query_index: u32) {
        self.render_pass
            .get_render_pass_mut()
            .write_timestamp(query_set, query_index);
    }

    pub fn set_blend_constant(&mut self, color: &Color) {
        self.render_pass
            .get_render_pass_mut()
            .set_blend_constant(*color);
    }

    pub fn pop_debug_group(&mut self) {
        self.render_pass.get_render_pass_mut().pop_debug_group();
    }

    pub fn push_debug_group(&mut self, label: &str) {
        self.render_pass
            .get_render_pass_mut()
            .push_debug_group(label);
    }

    pub fn insert_debug_marker(&mut self, label: &str) {
        self.render_pass
            .get_render_pass_mut()
            .insert_debug_marker(label);
    }

    pub fn set_viewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        self.render_pass
            .get_render_pass_mut()
            .set_viewport(x, y, width, height, min_depth, max_depth);
    }

    pub fn set_immediates(&mut self, offset: u32, data: &[u8]) {
        self.render_pass
            .get_render_pass_mut()
            .set_immediates(offset, data);
    }

    pub fn set_stencil_reference(&mut self, reference: u32) {
        self.render_pass
            .get_render_pass_mut()
            .set_stencil_reference(reference);
    }

    pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.render_pass
            .get_render_pass_mut()
            .set_scissor_rect(x, y, width, height);
    }

    pub fn multi_draw_indexed_indirect_count(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
        count_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        count_offset: u64,
        max_count: u32,
    ) {
        let indirect_buffer = self
            .pass_context
            .resource_table
            .get_resource(indirect_buffer_ref);

        let count_buffer = self
            .pass_context
            .resource_table
            .get_resource(count_buffer_ref);

        self.render_pass
            .get_render_pass_mut()
            .multi_draw_indexed_indirect_count(
                &indirect_buffer.resource,
                indirect_offset,
                &count_buffer.resource,
                count_offset,
                max_count,
            );
    }

    pub fn multi_draw_indexed_indirect(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
        count: u32,
    ) {
        let indirect_buffer = self
            .pass_context
            .resource_table
            .get_resource(indirect_buffer_ref);

        self.render_pass
            .get_render_pass_mut()
            .multi_draw_indexed_indirect(&indirect_buffer.resource, indirect_offset, count);
    }

    pub fn multi_draw_indirect_count(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
        count_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        count_offset: u64,
        max_count: u32,
    ) {
        let indirect_buffer = self
            .pass_context
            .resource_table
            .get_resource(indirect_buffer_ref);

        let count_buffer = self
            .pass_context
            .resource_table
            .get_resource(count_buffer_ref);

        self.render_pass
            .get_render_pass_mut()
            .multi_draw_indirect_count(
                &indirect_buffer.resource,
                indirect_offset,
                &count_buffer.resource,
                count_offset,
                max_count,
            );
    }

    pub fn draw_indexed_indirect(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
    ) {
        let indirect_buffer = self
            .pass_context
            .resource_table
            .get_resource(indirect_buffer_ref);

        self.render_pass
            .get_render_pass_mut()
            .draw_indexed_indirect(&indirect_buffer.resource, indirect_offset);
    }

    pub fn multi_draw_indirect(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
        count: u32,
    ) {
        let indirect_buffer = self
            .pass_context
            .resource_table
            .get_resource(indirect_buffer_ref);

        self.render_pass.get_render_pass_mut().multi_draw_indirect(
            &indirect_buffer.resource,
            indirect_offset,
            count,
        );
    }

    pub fn draw_indirect(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
    ) {
        let indirect_buffer = self
            .pass_context
            .resource_table
            .get_resource(indirect_buffer_ref);

        self.render_pass
            .get_render_pass_mut()
            .draw_indirect(&indirect_buffer.resource, indirect_offset);
    }

    pub fn set_bind_group(&mut self, index: u32, bind_group: &TransientBindGroup, offsets: &[u32]) {
        let bind_group = bind_group.create_bind_group(self.pass_context);

        self.render_pass
            .get_render_pass_mut()
            .set_bind_group(index, &bind_group, offsets);
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.render_pass
            .get_render_pass_mut()
            .draw_indexed(indices, base_vertex, instances);
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.render_pass
            .get_render_pass_mut()
            .draw(vertices, instances);
    }

    pub fn set_render_pipeline(&mut self, pipeline: &RenderPipeline) {
        self.render_pass
            .get_render_pass_mut()
            .set_pipeline(pipeline);
    }

    pub fn set_vertex_buffer(
        &mut self,
        slot: u32,
        buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        offset: u64,
        size: u64,
    ) {
        let buffer = self.pass_context.resource_table.get_resource(buffer_ref);
        self.render_pass
            .get_render_pass_mut()
            .set_vertex_buffer(slot, buffer.resource.slice(offset..(offset + size)));
    }

    pub fn set_index_buffer(
        &mut self,
        buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        index_format: IndexFormat,
        offset: u64,
        size: u64,
    ) {
        let buffer = self.pass_context.resource_table.get_resource(buffer_ref);

        self.render_pass
            .get_render_pass_mut()
            .set_index_buffer(buffer.resource.slice(offset..(offset + size)), index_format);
    }
}
