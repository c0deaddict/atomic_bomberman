mod ani;
mod pcx;
mod rss;

use ani::AniAssetLoader;
use pcx::PcxAssetLoader;
use rss::RssAssetLoader;

use bevy::prelude::*;

/// Adds support for loading custom assets.
#[derive(Default)]
pub struct CustomAssetsPlugin;

impl Plugin for CustomAssetsPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_asset_loader::<PcxAssetLoader>()
            .init_asset_loader::<RssAssetLoader>()
            .init_asset_loader::<AniAssetLoader>();
    }
}
