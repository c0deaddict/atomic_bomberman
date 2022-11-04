use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};

#[derive(Default)]
pub struct ColorRemapTableAssetLoader;

// 256 color mappings + 3 bytes unknown.
const TABLE_SIZE: usize = 259;

#[derive(Default, TypeUuid)]
#[uuid = "73eb678b-2b1c-4042-b2de-2f349b345845"]
pub struct ColorRemapTable {
    data: Vec<u8>,
}

impl ColorRemapTable {
    fn new(bytes: &[u8]) -> Result<ColorRemapTable> {
        if bytes.len() != TABLE_SIZE {
            bail!(
                "Expected color remap table to be exactly {} bytes",
                TABLE_SIZE
            );
        }

        Ok(ColorRemapTable { data: bytes.into() })
    }
}

impl AssetLoader for ColorRemapTableAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let table = ColorRemapTable::new(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(table));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["rmp", "RMP"]
    }
}
