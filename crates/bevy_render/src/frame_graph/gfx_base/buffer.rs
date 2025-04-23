use crate::render_resource::Buffer;

#[derive(Clone, PartialEq, Eq, Default, Hash)]
pub struct BufferInfo {}

pub struct FrameGraphBuffer {
    pub value: Buffer,
    pub desc: BufferInfo,
}
