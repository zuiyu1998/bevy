use std::{mem::take, ops::Range};

use wgpu::{IndexFormat, RenderPipeline};

use crate::frame_graph::{
    PassNodeBuilderExt, RenderPass, ResourceHandle, ResourceMaterial, ResourceRead, ResourceRef,
    ResourceWrite, TransientBuffer, TransientRenderPassColorAttachment, TransientResource,
};

use super::{PassBuilder, RenderPassExt};

pub struct RenderPassBuilder<'a, 'b> {
    render_pass: RenderPass,
    pass_builder: &'b mut PassBuilder<'a>,
}

impl Drop for RenderPassBuilder<'_, '_> {
    fn drop(&mut self) {
        self.finish();
    }
}

impl PassNodeBuilderExt for RenderPassBuilder<'_, '_> {
    fn read_material<M: ResourceMaterial>(
        &mut self,
        material: &M,
    ) -> ResourceRef<M::ResourceType, ResourceRead> {
        self.pass_builder.read_material(material)
    }

    fn write_material<M: ResourceMaterial>(
        &mut self,
        material: &M,
    ) -> ResourceRef<M::ResourceType, ResourceWrite> {
        self.pass_builder.write_material(material)
    }

    fn read<ResourceType: TransientResource>(
        &mut self,
        resource_handle: ResourceHandle<ResourceType>,
    ) -> ResourceRef<ResourceType, ResourceRead> {
        self.pass_builder.read(resource_handle)
    }

    fn write<ResourceType: TransientResource>(
        &mut self,
        resource_handle: ResourceHandle<ResourceType>,
    ) -> ResourceRef<ResourceType, ResourceWrite> {
        self.pass_builder.write(resource_handle)
    }
}

impl<'a, 'b> RenderPassBuilder<'a, 'b> {
    pub fn new(pass_builder: &'b mut PassBuilder<'a>, name: &str) -> Self {
        let mut render_pass = RenderPass::default();
        render_pass.set_pass_name(name);

        Self {
            render_pass,
            pass_builder,
        }
    }

    pub fn draw_indexed(
        &mut self,
        indices: Range<u32>,
        base_vertex: i32,
        instances: Range<u32>,
    ) -> &mut Self {
        self.render_pass
            .draw_indexed(indices, base_vertex, instances);

        self
    }

    pub fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>) -> &mut Self {
        self.render_pass.draw(vertices, instances);

        self
    }

    pub fn add_color_attachment(
        &mut self,
        color_attachment: TransientRenderPassColorAttachment,
    ) -> &mut Self {
        self.render_pass
            .add_color_attachment(Some(color_attachment));
        self
    }

    pub fn set_render_pipeline(&mut self, pipeline: &RenderPipeline) -> &mut Self {
        self.render_pass.set_render_pipeline(pipeline.clone());
        self
    }

    pub fn set_index_buffer(
        &mut self,
        buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        index_format: IndexFormat,
        offset: u64,
        size: u64,
    ) -> &mut Self {
        self.render_pass
            .set_index_buffer(buffer_ref, index_format, offset, size);

        self
    }

    pub fn set_vertex_buffer(
        &mut self,
        slot: u32,
        buffer_ref: &ResourceRef<TransientBuffer, ResourceRead>,
        offset: u64,
        size: u64,
    ) -> &mut Self {
        self.render_pass
            .set_vertex_buffer(slot, buffer_ref, offset, size);
        self
    }

    pub fn create_render_pass_builder(&mut self) -> &mut Self {
        self.finish();

        self
    }

    fn finish(&mut self) {
        let render_pass = take(&mut self.render_pass);
        self.pass_builder.push(render_pass);
    }
}
