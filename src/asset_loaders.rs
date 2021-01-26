mod color_palette;
mod pcx_image;
mod raw_sound;
mod scheme;

use crate::animation::{Animation, AnimationAssetLoader, AnimationBundle};
use color_palette::ColorPaletteAssetLoader;
use pcx_image::PcxImageAssetLoader;
use raw_sound::RawSoundAssetLoader;
use scheme::SchemeAssetLoader;

pub use scheme::{Cell, Grid, Powerup, PowerupInfo, Scheme};

use bevy::prelude::*;

/// Adds support for loading custom assets.
#[derive(Default)]
pub struct CustomAssetLoaders;

impl Plugin for CustomAssetLoaders {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<AnimationBundle>()
            .add_asset::<Animation>()
            .init_asset_loader::<AnimationAssetLoader>()
            .init_asset_loader::<PcxImageAssetLoader>()
            .init_asset_loader::<RawSoundAssetLoader>()
            .init_asset_loader::<ColorPaletteAssetLoader>()
            .add_asset::<Scheme>()
            .init_asset_loader::<SchemeAssetLoader>();
    }
}
