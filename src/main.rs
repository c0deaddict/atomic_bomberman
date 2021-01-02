use bevy::prelude::*;
use atomic_bomberman::assets::CustomAssetsPlugin;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(CustomAssetsPlugin)
        .init_resource::<State>()        
        .add_startup_system(setup.system())
        .add_startup_system(load_and_play_audio.system())        
        .add_system(print_on_load.system())
        .run();
}

#[derive(Default)]
struct State {
    handle: Handle<Texture>,
    printed: bool,
}

fn setup(mut state: ResMut<State>, asset_server: Res<AssetServer>) {
    state.handle = asset_server.load("data/RES/MAINMENU.PCX");
}

fn load_and_play_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    let music = asset_server.load("data/SOUND/MENU.RSS");
    audio.play(music);
}

fn print_on_load(mut state: ResMut<State>, custom_assets: ResMut<Assets<Texture>>) {
    let custom_asset = custom_assets.get(&state.handle);
    if state.printed || custom_asset.is_none() {
        return;
    }

    println!("Custom asset loaded: {:?}", custom_asset.unwrap());
    state.printed = true;
}
