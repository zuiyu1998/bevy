use std::collections::HashMap;

use super::RawResourceNodeHandle;

#[derive(Default)]
pub struct ResourceBoard {
    resources: HashMap<String, RawResourceNodeHandle>,
}

impl ResourceBoard {
    pub fn put(&mut self, name: &str, handle: RawResourceNodeHandle) {
        self.resources.insert(name.to_owned(), handle);
    }

    pub fn get(&self, name: &str) -> Option<&RawResourceNodeHandle> {
        self.resources.get(name)
    }
}
