use std::collections::HashMap;

use super::{AnyTransientResource, AnyTransientResourceDescriptor};

#[derive(Default)]
pub struct TransientResourceCache {
    resources: HashMap<AnyTransientResourceDescriptor, Vec<AnyTransientResource>>,
}

impl TransientResourceCache {
    pub fn get_resource(
        &mut self,
        desc: &AnyTransientResourceDescriptor,
    ) -> Option<AnyTransientResource> {
        if let Some(entry) = self.resources.get_mut(desc) {
            entry.pop()
        } else {
            None
        }
    }

    pub fn insert_resource(
        &mut self,
        desc: AnyTransientResourceDescriptor,
        resource: AnyTransientResource,
    ) {
        if let Some(entry) = self.resources.get_mut(&desc) {
            entry.push(resource);
        } else {
            self.resources.insert(desc, vec![resource]);
        }
    }
}
