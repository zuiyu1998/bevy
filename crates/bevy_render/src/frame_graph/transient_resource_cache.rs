use std::collections::HashMap;
use bevy_ecs::resource::Resource;

use crate::frame_graph::{AnyResource, AnyResourceDescriptor};

#[derive(Default, Resource)]
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
