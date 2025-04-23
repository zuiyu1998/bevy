use std::{marker::PhantomData, sync::Arc};

use super::{PassNode, TypeHandle};

pub trait ImportToFrameGraph
where
    Self: Sized + Resource,
{
    fn import(self: Arc<Self>) -> ImportedVirtualResource;
}

#[derive(Clone)]
pub enum ImportedVirtualResource {}

#[derive(Clone)]
pub struct VirtualResource {
    pub info: ResourceInfo,
    pub state: ResourceState,
}

impl VirtualResource {
    pub fn new_setuped<ResourceType: Resource>(
        desc: ResourceType::Descriptor,
        name: &str,
        handle: TypeHandle<VirtualResource>,
    ) -> VirtualResource {
        VirtualResource {
            info: ResourceInfo::new(name, handle),
            state: ResourceState::Setuped(desc.into()),
        }
    }

    #[allow(unreachable_code)]
    #[allow(unused_variables)]
    pub fn new_imported<ResourceType: Resource>(
        name: &str,
        handle: TypeHandle<VirtualResource>,
        desc: ResourceType::Descriptor,
        imported_resource: ImportedVirtualResource,
    ) -> VirtualResource {
        let info = ResourceInfo::new(name, handle);

        let state = ResourceState::imported(ImportedResourceState::new(
            desc.into(),
            imported_resource.clone(),
        ));

        VirtualResource { info, state }
    }
}

#[derive(Clone)]
pub struct ResourceInfo {
    pub name: String,
    pub handle: TypeHandle<VirtualResource>,
    version: u32,
    pub first_use_pass: Option<TypeHandle<PassNode>>,
    pub last_user_pass: Option<TypeHandle<PassNode>>,
}

impl ResourceInfo {
    pub fn new(name: &str, handle: TypeHandle<VirtualResource>) -> Self {
        ResourceInfo {
            name: name.to_string(),
            handle,
            version: 0,
            first_use_pass: None,
            last_user_pass: None,
        }
    }
}

impl ResourceInfo {
    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn new_version(&mut self) {
        self.version += 1
    }

    pub fn update_lifetime(&mut self, handle: TypeHandle<PassNode>) {
        if self.first_use_pass.is_none() {
            self.first_use_pass = Some(handle);
        }

        self.last_user_pass = Some(handle)
    }
}

#[derive(Clone)]
pub struct ImportedResourceState {
    pub desc: AnyResourceDescriptor,
    pub resource: ImportedVirtualResource,
}

impl ImportedResourceState {
    pub fn new(desc: AnyResourceDescriptor, resource: ImportedVirtualResource) -> Self {
        Self { desc, resource }
    }
}

#[derive(Clone)]
pub enum ResourceState {
    Setuped(AnyResourceDescriptor),
    Imported(ImportedResourceState),
}

impl ResourceState {
    pub fn imported(state: ImportedResourceState) -> Self {
        ResourceState::Imported(state)
    }

    pub fn setuped(desc: AnyResourceDescriptor) -> Self {
        ResourceState::Setuped(desc)
    }
}

#[derive(Clone)]
pub enum AnyResourceDescriptor {}

pub enum AnyResource {}

pub trait Resource: 'static {
    type Descriptor: ResourceDescriptor;

    fn borrow_resource(res: &AnyResource) -> &Self;

    fn get_desc(&self) -> &Self::Descriptor;
}

pub trait ResourceDescriptor: 'static + Clone + Into<AnyResourceDescriptor> {
    type Resource: Resource;
}

pub trait TypeEquals {
    type Other;
    fn same(value: Self) -> Self::Other;
}

impl<T: Sized> TypeEquals for T {
    type Other = Self;
    fn same(value: Self) -> Self::Other {
        value
    }
}

pub struct ResourceRef<ResourceType, ViewType> {
    pub index: TypeHandle<VirtualResource>,
    _marker: PhantomData<(ResourceType, ViewType)>,
}

impl<ResourceType, ViewType> ResourceRef<ResourceType, ViewType> {
    pub fn new(index: TypeHandle<VirtualResource>) -> Self {
        Self {
            index,
            _marker: PhantomData,
        }
    }
}

pub trait GpuView {}

pub struct GpuRead;

impl GpuView for GpuRead {}

pub struct GpuWrite;

impl GpuView for GpuWrite {}
