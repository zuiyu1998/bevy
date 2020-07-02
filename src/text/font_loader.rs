use super::Font;
use anyhow::Result;
use crate::asset::AssetLoader;
use std::path::Path;

#[derive(Default)]
pub struct FontLoader;

impl AssetLoader<Font> for FontLoader {
    fn from_bytes(&self, _asset_path: &Path, bytes: Vec<u8>) -> Result<Font> {
        Ok(Font::try_from_bytes(bytes)?)
    }
    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["ttf"];
        EXTENSIONS
    }
}
