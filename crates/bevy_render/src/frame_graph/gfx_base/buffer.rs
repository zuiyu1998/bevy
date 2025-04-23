use crate::render_resource::Buffer;

pub struct BufferInfo {}

pub struct FrameGraphBuffer {
    pub value: Buffer,
    pub desc: BufferInfo,
}
