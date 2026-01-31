mod draw_indexed_parameter;
mod draw_parameter;
mod set_bind_group_parameter;
mod set_index_buffer_parameter;
mod set_render_pipeline_parameter;
mod set_vertex_buffer_parameter;

use crate::frame_graph::{
    RenderPass, RenderPassCommand, ResourceRead, ResourceRef, TransientBindGroup, TransientBuffer,
};
use draw_indexed_parameter::*;
use draw_parameter::*;
use set_bind_group_parameter::*;
use set_index_buffer_parameter::*;
use set_render_pipeline_parameter::*;
use set_vertex_buffer_parameter::*;
use std::ops::Range;
use wgpu::{IndexFormat, RenderPipeline};

pub trait RenderPassExt {
    fn push<T: RenderPassCommand>(&mut self, value: T);

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
