use bevy_platform::collections::HashMap;

use super::{AnyTransientResource, TransientResource};

pub struct ResourceTable(HashMap<usize, AnyTransientResource>);

impl Default for ResourceTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceTable {
    pub fn new() -> Self {
        Self(HashMap::default())
    }

    pub fn insert(&mut self, index: usize, resource: AnyTransientResource) {
        self.0.insert(index, resource);
    }

    pub fn get_resource<ResourceType: TransientResource>(&self, index: &usize) -> &ResourceType {
        self.try_get_resource(index).expect("Resource not found")
    }

    pub fn try_get_resource<ResourceType: TransientResource>(
        &self,
        index: &usize,
    ) -> Option<&ResourceType> {
        Some(ResourceType::borrow_resource(self.0.get(index)?))
    }

    pub fn get(&self, index: &usize) -> Option<&AnyTransientResource> {
        self.0.get(index)
    }
}
