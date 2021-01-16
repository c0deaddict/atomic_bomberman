mod animation;
mod animation_list;
mod color_palette;
mod pcx_image;
mod raw_sound;
mod scheme;

use animation::AnimationAssetLoader;
use animation_list::AnimationListAssetLoader;
use color_palette::ColorPaletteAssetLoader;
use pcx_image::PcxImageAssetLoader;
use raw_sound::RawSoundAssetLoader;
use scheme::SchemeAssetLoader;

pub use animation::{Animation, AnimationBundle, Frame};
pub use animation_list::AnimationList;
pub use scheme::{Cell, Grid, Powerup, PowerupInfo, Scheme};

use bevy::prelude::*;

/// Adds support for loading custom assets.
#[derive(Default)]
pub struct CustomAssetLoaders;

impl Plugin for CustomAssetLoaders {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<AnimationBundle>()
            .init_asset_loader::<AnimationAssetLoader>()
            .add_asset::<AnimationList>()
            .init_asset_loader::<AnimationListAssetLoader>()
            .init_asset_loader::<PcxImageAssetLoader>()
            .init_asset_loader::<RawSoundAssetLoader>()
            .init_asset_loader::<ColorPaletteAssetLoader>()
            .add_asset::<Scheme>()
            .init_asset_loader::<SchemeAssetLoader>();
    }
}
