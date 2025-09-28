use core::marker::PhantomData;

use crate::frame_graph::{IndexHandle, PassNode, TransientResource, VirtualResource};

pub trait ResourceView {}

pub struct ResourceRead;

pub struct ResourceWrite;

impl ResourceView for ResourceRead {}

impl ResourceView for ResourceWrite {}

pub struct Ref<ResourceType: TransientResource, VieType> {
    resource_handle: ResourceHandle,
    desc: <ResourceType as TransientResource>::Descriptor,
    _marker: PhantomData<(ResourceType, VieType)>,
}

pub struct Handle<ResourceType: TransientResource> {
    resource_handle: ResourceHandle,
    desc: <ResourceType as TransientResource>::Descriptor,
    _marker: PhantomData<ResourceType>,
}

impl<ResourceType: TransientResource, ViewType> Ref<ResourceType, ViewType> {
    pub fn new(
        index: IndexHandle<ResourceNode>,
        version: u32,
        desc: <ResourceType as TransientResource>::Descriptor,
    ) -> Self {
        Ref {
            resource_handle: ResourceHandle {
                index: index.clone(),
                version,
            },
            desc,
            _marker: PhantomData,
        }
    }

    pub fn get_resource_handle(&self) -> &ResourceHandle {
        &self.resource_handle
    }

    pub fn get_desc(&self) -> &<ResourceType as TransientResource>::Descriptor {
        &self.desc
    }
}

impl<ResourceType: TransientResource> Handle<ResourceType> {
    pub fn new(
        index: IndexHandle<ResourceNode>,
        version: u32,
        desc: <ResourceType as TransientResource>::Descriptor,
    ) -> Self {
        Handle {
            resource_handle: ResourceHandle {
                index: index.clone(),
                version,
            },
            desc,
            _marker: PhantomData,
        }
    }

    pub fn get_resource_handle(&self) -> &ResourceHandle {
        &self.resource_handle
    }

    pub fn get_desc(&self) -> &<ResourceType as TransientResource>::Descriptor {
        &self.desc
    }
}

pub struct ResourceHandle {
    index: IndexHandle<ResourceNode>,
    version: u32,
}

impl ResourceHandle {
    pub fn new(index: IndexHandle<ResourceNode>, version: u32) -> Self {
        Self { index, version }
    }

    pub fn handle(&self) -> &IndexHandle<ResourceNode> {
        &self.index
    }

    pub fn index(&self) -> usize {
        self.index.index()
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

pub struct ResourceRequese {
    pub handle: IndexHandle<ResourceNode>,
    pub resource: VirtualResource,
}

pub struct ResourceRelease {
    pub handle: IndexHandle<ResourceNode>,
}

pub struct ResourceNode {
    pub handle: IndexHandle<ResourceNode>,
    pub name: String,
    pub first_use_pass: Option<IndexHandle<PassNode>>,
    pub last_user_pass: Option<IndexHandle<PassNode>>,
    version: u32,
    pub resource: VirtualResource,
}

impl ResourceNode {
    pub fn new(name: &str, handle: IndexHandle<ResourceNode>, resource: VirtualResource) -> Self {
        ResourceNode {
            name: name.to_string(),
            handle,
            version: 0,
            first_use_pass: None,
            last_user_pass: None,
            resource,
        }
    }

    pub fn request(&self) -> ResourceRequese {
        ResourceRequese {
            handle: self.handle.clone(),
            resource: self.resource.clone(),
        }
    }

    pub fn get_handle<ResourceType: TransientResource>(&self) -> Handle<ResourceType> {
        let desc = self.get_desc::<ResourceType>().clone();
        Handle::new(self.handle.clone(), self.version, desc)
    }
    pub fn get_desc<ResourceType: TransientResource>(&self) -> ResourceType::Descriptor {
        self.resource.get_desc::<ResourceType>()
    }

    pub fn release(&self) -> ResourceRelease {
        ResourceRelease {
            handle: self.handle.clone(),
        }
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    pub fn new_version(&mut self) {
        self.version += 1;
    }

    pub fn update_lifetime(&mut self, handle: IndexHandle<PassNode>) {
        if self.first_use_pass.is_none() {
            self.first_use_pass = Some(handle.clone());
        }

        self.last_user_pass = Some(handle);
    }
}
