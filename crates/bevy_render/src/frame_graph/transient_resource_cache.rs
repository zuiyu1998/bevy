use std::collections::HashMap;

use crate::frame_graph::{AnyResource, AnyResourceDescriptor};

#[derive(Default)]
pub struct TransientResourceCache {
    resources: HashMap<AnyResourceDescriptor, Vec<AnyResource>>,
}

impl TransientResourceCache {
    pub fn get_resource(&mut self, desc: &AnyResourceDescriptor) -> Option<AnyResource> {
        if let Some(entry) = self.resources.get_mut(desc) {
            entry.pop()
        } else {
            None
        }
    }

    pub fn insert_resource(&mut self, desc: AnyResourceDescriptor, resource: AnyResource) {
        if let Some(entry) = self.resources.get_mut(&desc) {
            entry.push(resource);
        } else {
            self.resources.insert(desc, vec![resource]);
        }
    }
}
