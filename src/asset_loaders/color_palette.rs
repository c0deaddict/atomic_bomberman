use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};

#[derive(Default)]
pub struct ColorPaletteAssetLoader;

const PALETTE_SIZE: usize = 256 * 3 + 32768; // 256 RGB colors + 2^15 mapping.

#[derive(Default, TypeUuid)]
#[uuid = "398f6778-81f8-4293-a3b9-3b72ba4b5b55"]
pub struct ColorPalette {
    data: Vec<u8>,
}

impl ColorPalette {
    fn new(bytes: &[u8]) -> Result<ColorPalette> {
        if bytes.len() != PALETTE_SIZE {
            bail!(
                "Expected color palette to be exactly {} bytes",
                PALETTE_SIZE
            );
        }

        Ok(ColorPalette { data: bytes.into() })
    }
}

impl AssetLoader for ColorPaletteAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let palette = ColorPalette::new(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(palette));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pal", "PAL"]
    }
}
