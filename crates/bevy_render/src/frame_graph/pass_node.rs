use crate::{IndexHandle, Pass, RawResourceHandle, ResourceNode};

pub struct PassNode {
    pub name: String,
    pub index: IndexHandle<PassNode>,
    pub writes: Vec<RawResourceHandle>,
    pub reads: Vec<RawResourceHandle>,
    pub resource_request_array: Vec<IndexHandle<ResourceNode>>,
    pub resource_release_array: Vec<IndexHandle<ResourceNode>>,
    pub pass: Option<Pass>,
}

impl PassNode {
    pub fn new(name: &str, index: IndexHandle<PassNode>) -> Self {
        Self {
            name: name.to_string(),
            index,
            writes: Default::default(),
            reads: Default::default(),
            resource_request_array: Default::default(),
            resource_release_array: Default::default(),
            pass: Default::default(),
        }
    }
}
