use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    render::texture::{Extent3d, Texture, TextureDimension, TextureFormat},
    prelude::*,
    utils::BoxedFuture,    
};

/// Convert Bomberman specific ANI format to a sprite sheet.
#[derive(Default)]
pub struct AniAssetLoader;

impl AssetLoader for AniAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            // https://github.com/mmatyas/ab_aniex/blob/master/src/AniFile.cpp
            // https://github.com/image-rs/image/blob/master/src/codecs/bmp/decoder.rs
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
        &["ani", "ANI"]
    }
}
