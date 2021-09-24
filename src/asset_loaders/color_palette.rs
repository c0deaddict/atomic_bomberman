use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    render::texture::{Extent3d, Texture, TextureDimension, TextureFormat},
    utils::BoxedFuture,
};

#[derive(Default)]
pub struct ColorPaletteAssetLoader;

impl AssetLoader for ColorPaletteAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let rmp = std::fs::read("/home/jos/tmp/BM95/6.RMP").unwrap();

            let width = 128;
            let height = 256;
            let mut image_data = vec![0; 4 * width * height];
            for i in 0..width * height {
                let idx = rmp[bytes[i + 768] as usize] as usize;
                let r = bytes[idx * 3 + 0] << 2;
                let g = bytes[idx * 3 + 1] << 2;
                let b = bytes[idx * 3 + 2] << 2;
                if i == 0x3e0 || i == 0x2a0 {
                    println!("{:04x}: {:02x}{:02x}{:02x}", i, r, g, b);
                }
                image_data[i * 4 + 0] = r;
                image_data[i * 4 + 1] = g;
                image_data[i * 4 + 2] = b;
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
