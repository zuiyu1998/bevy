use crate::{
    frame_graph::{ResourceTable, ResourceView, Result},
    render_resource::BindGroupLayout,
};

pub enum BindingResourceRef {}

pub struct BindGroupEntryRef {
    pub binding: u32,
    pub resource: BindingResourceRef,
}

pub struct BindGroupRef {
    pub layout: BindGroupLayout,
    pub entries: Vec<BindGroupEntryRef>,
}

pub struct BindGroupView<'a> {
    pub layout: &'a BindGroupLayout,
    pub entries: &'a [wgpu::BindGroupEntry<'a>],
}

impl<'a> ResourceView<'a> for BindGroupView<'a> {
    type ViewRef = BindGroupRef;

    fn prepare_view(_resource_table: &ResourceTable, _view_ref: &Self::ViewRef) -> Result<Self> {
        todo!()
    }
}
