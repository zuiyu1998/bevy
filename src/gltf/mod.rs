mod loader;
pub use loader::*;

use crate::app::{AppBuilder, AppPlugin};
use crate::asset::AddAsset;
use crate::render::mesh::Mesh;

#[derive(Default)]
pub struct GltfPlugin;

impl AppPlugin for GltfPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset_loader::<Mesh, GltfLoader>();
    }
}
