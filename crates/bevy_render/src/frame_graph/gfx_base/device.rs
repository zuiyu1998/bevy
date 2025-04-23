use crate::{frame_graph::{AnyResource, AnyResourceDescriptor}, renderer::RenderDevice};

use super::FrameGraphCommandBuffer;

impl DeviceTrait for RenderDevice {
    fn create_command_buffer(&self) -> FrameGraphCommandBuffer {
        FrameGraphCommandBuffer::new(self)
    }
    
    fn create(&self, _desc: &AnyResourceDescriptor) -> AnyResource {
        todo!()
    }
}

pub trait DeviceTrait: 'static + Sync + Send + Clone {
    fn create_command_buffer(&self) -> FrameGraphCommandBuffer;

    fn create(&self, desc: &AnyResourceDescriptor) -> AnyResource;
}