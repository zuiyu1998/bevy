use wgpu::BindGroup;

#[derive(Debug, Clone)]
pub struct GpuBindGroup(BindGroup);

impl GpuBindGroup {
    pub fn get_bind_group(&self) -> &BindGroup {
        &self.0
    }
}

impl From<BindGroup> for GpuBindGroup {
    fn from(value: BindGroup) -> Self {
        GpuBindGroup(value)
    }
}
