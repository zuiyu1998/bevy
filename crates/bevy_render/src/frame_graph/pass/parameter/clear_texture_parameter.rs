use wgpu::ImageSubresourceRange;

use crate::frame_graph::{
    EncoderCommand, PassContext, ResourceRef, ResourceWrite, TransientTexture,
};

pub struct ClearTextureParameter {
    pub texture: ResourceRef<TransientTexture, ResourceWrite>,
    pub subresource_range: ImageSubresourceRange,
}

impl EncoderCommand for ClearTextureParameter {
    fn execute(&self, context: &mut PassContext) {
        let texture: &TransientTexture = context.resource_table.get_resource(&self.texture);

        context
            .command_encoder
            .clear_texture(&texture.resource, &self.subresource_range);
    }
}
