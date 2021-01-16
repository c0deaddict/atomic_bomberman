use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    render::texture::{Extent3d, Texture, TextureDimension, TextureFormat},
    utils::BoxedFuture,
};
use byteorder::{ReadBytesExt, LE};
use std::io::{Cursor, Read};

#[derive(Default)]
pub struct ColorPaletteAssetLoader;

impl AssetLoader for ColorPaletteAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let width = 150;
            let height = bytes.len() / (3 * width);

            let mut image_data = vec![0; 4 * width * height];
            let j = 2;
            for i in 0..width * height {
                image_data[i * 4 + 0] = bytes[j + i * 3 + 0];
                image_data[i * 4 + 1] = bytes[j + i * 3 + 1];
                image_data[i * 4 + 2] = bytes[j + i * 3 + 2];
                image_data[i * 4 + 3] = 255;
            }

            let texture = Texture::new(
                Extent3d::new(width as u32, height as u32, 1),
                TextureDimension::D2,
                image_data,
                TextureFormat::Rgba8UnormSrgb,
            );

            load_context.set_default_asset(LoadedAsset::new(texture));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pal", "PAL"]
    }
}
