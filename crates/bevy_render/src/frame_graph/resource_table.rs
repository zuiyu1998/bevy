use super::{GpuRead, ResourceRef, Result};

#[derive(Default)]
pub struct ResourceTable;

impl ResourceTable {
    pub fn get_resource<ResourceType>(
        &self,
        _resource_ref: &ResourceRef<ResourceType, GpuRead>,
    ) -> Option<&ResourceType> {
        todo!()
    }
}

pub trait ResourceView: Sized {
    type ViewRef: 'static;

    fn prepare_view(resource_table: &ResourceTable, view_ref: &Self::ViewRef) -> Result<Self>;
}
