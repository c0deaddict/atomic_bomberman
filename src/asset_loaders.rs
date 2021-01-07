mod animation;
mod pcx_image;
mod raw_sound;

use animation::AnimationAssetLoader;
pub use animation::AnimationBundle;
use pcx_image::PcxImageAssetLoader;
use raw_sound::RawSoundAssetLoader;

use bevy::prelude::*;

/// Adds support for loading custom assets.
#[derive(Default)]
pub struct CustomAssetLoaders;

impl Plugin for CustomAssetLoaders {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<AnimationBundle>()
            .init_asset_loader::<PcxImageAssetLoader>()
            .init_asset_loader::<RawSoundAssetLoader>()
            .init_asset_loader::<AnimationAssetLoader>();
    }
}
