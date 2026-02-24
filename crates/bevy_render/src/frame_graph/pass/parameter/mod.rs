mod begin_pipeline_statistics_query_parameter;
mod clear_texture_parameter;
mod draw_indexed_indirect_parameter;
mod draw_indexed_parameter;
mod draw_indirect_parameter;
mod draw_parameter;
mod end_pipeline_statistics_query_parameter;
mod insert_debug_marker_parameter;
mod multi_draw_indexed_indirect_count_parameter;
mod multi_draw_indexed_indirect_parameter;
mod multi_draw_indirect_count_parameter;
mod multi_draw_indirect_parameter;
mod pop_debug_group_parameter;
mod push_debug_group_parameter;
mod set_bind_group_parameter;
mod set_blend_constant_parameter;
mod set_immediates_parameter;
mod set_index_buffer_parameter;
mod set_render_pipeline_parameter;
mod set_scissor_rect_parameter;
mod set_stencil_reference_parameter;
mod set_vertex_buffer_parameter;
mod set_viewport_parameter;
mod write_timestamp_parameter;

use crate::frame_graph::{
    EncoderCommand, EncoderCommands, RenderPass, RenderPassCommand, ResourceRead, ResourceRef,
    ResourceWrite, TransientBindGroup, TransientBuffer, TransientTexture,
};
use core::ops::Range;

use begin_pipeline_statistics_query_parameter::*;
use clear_texture_parameter::*;
use draw_indexed_indirect_parameter::*;
use draw_indexed_parameter::*;
use draw_indirect_parameter::*;
use draw_parameter::*;
use end_pipeline_statistics_query_parameter::*;
use insert_debug_marker_parameter::*;
use multi_draw_indexed_indirect_count_parameter::*;
use multi_draw_indexed_indirect_parameter::*;
use multi_draw_indirect_count_parameter::*;
use multi_draw_indirect_parameter::*;
use pop_debug_group_parameter::*;
use push_debug_group_parameter::*;
use set_bind_group_parameter::*;
use set_blend_constant_parameter::*;
use set_immediates_parameter::*;
use set_index_buffer_parameter::*;
use set_render_pipeline_parameter::*;
use set_scissor_rect_parameter::*;
use set_stencil_reference_parameter::*;
use set_vertex_buffer_parameter::*;
use set_viewport_parameter::*;
use write_timestamp_parameter::*;

use wgpu::{Color, ImageSubresourceRange, IndexFormat, QuerySet, RenderPipeline};

pub trait EncoderExt {
    fn push<T: EncoderCommand>(&mut self, value: T);

    fn clear_texture(
        &mut self,
        texture: &ResourceRef<TransientTexture, ResourceWrite>,
        subresource_range: ImageSubresourceRange,
    ) {
        self.push(ClearTextureParameter {
            texture: texture.clone(),
            subresource_range,
        });
    }
}

impl EncoderExt for EncoderCommands {
    fn push<T: EncoderCommand>(&mut self, value: T) {
        self.commands.push(Box::new(value));
    }
}

pub trait RenderPassExt {
    fn push<T: RenderPassCommand>(&mut self, value: T);

    fn end_pipeline_statistics_query(&mut self) {
        self.push(EndPipelineStatisticsQueryParameter);
    }

    fn begin_pipeline_statistics_query(&mut self, query_set: &QuerySet, query_index: u32) {
        self.push(BeginPipelineStatisticsQueryParameter {
            query_index,
            query_set: query_set.clone(),
        });
    }

    fn write_timestamp(&mut self, query_set: &QuerySet, query_index: u32) {
        self.push(WriteTimestampParameter {
            query_index,
            query_set: query_set.clone(),
        });
    }

    fn set_blend_constant(&mut self, color: &Color) {
        self.push(SetBlendConstantParameter { color: *color });
    }

    fn pop_debug_group(&mut self) {
        self.push(PopDebugGroupParameter);
    }

    fn push_debug_group(&mut self, label: &str) {
        self.push(PushDebugGroupParameter {
            label: label.to_string(),
        });
    }

    fn insert_debug_marker(&mut self, label: &str) {
        self.push(InsertDebugMarkerParameter {
            label: label.to_string(),
        });
    }

    fn set_viewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        self.push(SetViewportParameter {
            x,
            y,
            width,
            height,
            max_depth,
            min_depth,
        });
    }

    fn set_immediates(&mut self, offset: u32, data: &[u8]) {
        self.push(SetImmediatesParameter {
            offset,
            data: data.to_vec(),
        });
    }

    fn set_stencil_reference(&mut self, reference: u32) {
        self.push(SetStencilReferenceParameter { reference });
    }

    fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        self.push(SetScissorRectParameter {
            x,
            y,
            width,
            height,
        });
    }

    fn multi_draw_indexed_indirect_count(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
        count_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        count_offset: u64,
        max_count: u32,
    ) {
        self.push(MultiDrawIndexedIndirectCountParameter {
            indirect_buffer_ref: indirect_buffer_ref.clone(),
            indirect_offset,
            count_buffer_ref: count_buffer_ref.clone(),
            count_offset,
            max_count,
        });
    }

    fn multi_draw_indexed_indirect(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
        count: u32,
    ) {
        self.push(MultiDrawIndexedIndirectParameter {
            indirect_buffer_ref: indirect_buffer_ref.clone(),
            indirect_offset,
            count,
        });
    }

    fn multi_draw_indirect_count(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
        count_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        count_offset: u64,
        max_count: u32,
    ) {
        self.push(MultiDrawIndirectCountParameter {
            indirect_buffer_ref: indirect_buffer_ref.clone(),
            indirect_offset,
            count_buffer_ref: count_buffer_ref.clone(),
            count_offset,
            max_count,
        });
    }

    fn draw_indirect(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
    ) {
        self.push(DrawIndirectParameter {
            indirect_buffer_ref: indirect_buffer_ref.clone(),
            indirect_offset,
        });
    }

    fn multi_draw_indirect(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
        count: u32,
    ) {
        self.push(MultiDrawIndirectParameter {
            indirect_buffer_ref: indirect_buffer_ref.clone(),
            indirect_offset,
            count,
        });
    }

    fn draw_indexed_indirect(
        &mut self,
        indirect_buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        indirect_offset: u64,
    ) {
        self.push(DrawIndexedIndirectParameter {
            indirect_buffer_ref: indirect_buffer_ref.clone(),
            indirect_offset,
        });
    }

    fn set_bind_group(&mut self, index: u32, bind_group: &TransientBindGroup, offsets: &[u32]) {
        self.push(SetBindGroupParameter {
            index,
            bind_group: bind_group.clone(),
            offsets: offsets.to_vec(),
        });
    }

    fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32, instances: Range<u32>) {
        self.push(DrawIndexedParameter {
            indices,
            base_vertex,
            instances,
        });
    }

    fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) {
        self.push(DrawParameter {
            vertices,
            instances,
        });
    }

    fn set_render_pipeline(&mut self, pipeline: RenderPipeline) {
        self.push(SetRenderPipelineParameter { pipeline });
    }

    fn set_vertex_buffer(
        &mut self,
        slot: u32,
        buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        offset: u64,
        size: u64,
    ) {
        self.push(SetVertexBufferParameter {
            slot,
            buffer_ref: buffer_ref.clone(),
            offset,
            size,
        });
    }

    fn set_index_buffer(
        &mut self,
        buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        index_format: IndexFormat,
        offset: u64,
        size: u64,
    ) {
        self.push(SetIndexBufferParameter {
            buffer_ref: buffer_ref.clone(),
            index_format,
            offset,
            size,
        });
    }
}

impl RenderPassExt for RenderPass {
    fn push<T: RenderPassCommand>(&mut self, value: T) {
        self.commands.push(Box::new(value));
    }
}
