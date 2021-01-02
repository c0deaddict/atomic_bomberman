use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    render::texture::{Extent3d, Texture, TextureDimension, TextureFormat},
    prelude::*,
    utils::BoxedFuture,    
};

#[derive(Default)]
pub struct PcxAssetLoader;

impl AssetLoader for PcxAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            // let img = None;
            // TODO: parse PCX format and convert to Texture.
            // only support the formats used by BM95.
            // https://github.com/image-rs/image/blob/master/src/codecs/bmp/decoder.rs
            // https://en.wikipedia.org/wiki/PCX
            let width = 1;
            let height = 1;
            let data = vec![0; 4];
            let format = TextureFormat::Rgba8UnormSrgb;
            
            let texture = Texture::new(
                Extent3d::new(width, height, 1),
                TextureDimension::D2,
                data,
                format,
            );

            load_context.set_default_asset(LoadedAsset::new(texture));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["pcx", "PCX"]
    }
}
