use core::mem::take;

use crate::frame_graph::{
    graph::pass_node_builder::encoder_builder::EncoderBuilder, Pass, PassCommand,
    PassNodeBuilderExt, RenderPassBuilder, ResourceHandle, ResourceMaterial, ResourceRead,
    ResourceRef, ResourceWrite, TransientResource,
};

use super::PassNodeBuilder;

pub struct PassBuilder<'a> {
    pass_node_builder: PassNodeBuilder<'a>,
    pass: Pass,
}

impl Drop for PassBuilder<'_> {
    fn drop(&mut self) {
        let pass = take(&mut self.pass);
        self.pass_node_builder.set_pass(pass);
    }
}

impl<'a> PassNodeBuilderExt for PassBuilder<'a> {
    fn read_material<M: ResourceMaterial>(
        &mut self,
        material: &M,
    ) -> ResourceRef<M::ResourceType, ResourceRead> {
        self.pass_node_builder.read_material(material)
    }

    fn write_material<M: ResourceMaterial>(
        &mut self,
        material: &M,
    ) -> ResourceRef<M::ResourceType, ResourceWrite> {
        self.pass_node_builder.write_material(material)
    }

    fn read<ResourceType: TransientResource>(
        &mut self,
        resource_handle: ResourceHandle<ResourceType>,
    ) -> ResourceRef<ResourceType, ResourceRead> {
        self.pass_node_builder.read(resource_handle)
    }

    fn write<ResourceType: TransientResource>(
        &mut self,
        resource_handle: ResourceHandle<ResourceType>,
    ) -> ResourceRef<ResourceType, ResourceWrite> {
        self.pass_node_builder.write(resource_handle)
    }
}

impl<'a> PassBuilder<'a> {
    pub fn new(pass_node_builder: PassNodeBuilder<'a>) -> Self {
        PassBuilder {
            pass_node_builder,
            pass: Pass::default(),
        }
    }

    pub fn create_render_pass_builder(&mut self, name: &str) -> RenderPassBuilder<'a, '_> {
        RenderPassBuilder::new(self, name)
    }

    pub fn create_encoder_builder(&mut self) -> EncoderBuilder<'a, '_> {
        EncoderBuilder::new(self)
    }

    pub fn push<T: PassCommand>(&mut self, command: T) {
        self.pass.push(command);
    }
}
