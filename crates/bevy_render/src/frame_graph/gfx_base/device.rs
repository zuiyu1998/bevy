use crate::renderer::RenderDevice;

use super::FrameGraphCommandBuffer;

impl DeviceTrait for RenderDevice {
    fn create_command_buffer(&self) -> FrameGraphCommandBuffer {
        todo!()
    }
}

pub trait DeviceTrait: 'static + Sync + Send + Clone {
    fn create_command_buffer(&self) -> FrameGraphCommandBuffer;
}