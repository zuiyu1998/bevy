use core::mem::take;

use wgpu::ImageSubresourceRange;

use crate::frame_graph::{
    EncoderCommands, EncoderExt, PassBuilder, PassNodeBuilderExt, ResourceHandle, ResourceMaterial,
    ResourceRead, ResourceRef, ResourceWrite, TransientResource, TransientTexture,
};

pub struct EncoderBuilder<'a, 'b> {
    commands: EncoderCommands,
    pass_builder: &'b mut PassBuilder<'a>,
}

impl Drop for EncoderBuilder<'_, '_> {
    fn drop(&mut self) {
        self.finish();
    }
}

impl PassNodeBuilderExt for EncoderBuilder<'_, '_> {
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

impl<'a, 'b> EncoderBuilder<'a, 'b> {
    pub fn new(pass_builder: &'b mut PassBuilder<'a>) -> Self {
        let commands = EncoderCommands::default();

        Self {
            commands,
            pass_builder,
        }
    }

    pub fn clear_texture(
        &mut self,
        texture: &ResourceRef<TransientTexture, ResourceWrite>,
        subresource_range: ImageSubresourceRange,
    ) -> &mut Self {
        self.commands.clear_texture(texture, subresource_range);

        self
    }

    fn finish(&mut self) {
        let commands = take(&mut self.commands);
        self.pass_builder.push(commands);
    }
}
