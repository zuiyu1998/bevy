use super::{serde::SceneDeserializer, Scene};
use anyhow::Result;
use crate::app::FromResources;
use crate::asset::AssetLoader;
use bevy_property::PropertyTypeRegistry;
use crate::type_registry::TypeRegistry;
use legion::prelude::Resources;
use serde::de::DeserializeSeed;
use std::{
    path::Path,
    sync::{Arc, RwLock},
};

pub struct SceneLoader {
    property_type_registry: Arc<RwLock<PropertyTypeRegistry>>,
}

impl FromResources for SceneLoader {
    fn from_resources(resources: &Resources) -> Self {
        let type_registry = resources.get::<TypeRegistry>().unwrap();
        SceneLoader {
            property_type_registry: type_registry.property.clone(),
        }
    }
}

impl AssetLoader<Scene> for SceneLoader {
    fn from_bytes(&self, _asset_path: &Path, bytes: Vec<u8>) -> Result<Scene> {
        let registry = self.property_type_registry.read().unwrap();
        let mut deserializer = ron::de::Deserializer::from_bytes(&bytes)?;
        let scene_deserializer = SceneDeserializer {
            property_type_registry: &registry,
        };
        let scene = scene_deserializer.deserialize(&mut deserializer)?;
        Ok(scene)
    }
    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["scn"];
        EXTENSIONS
    }
}
