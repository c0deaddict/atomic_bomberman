use atomic_bomberman::assets::CustomAssetsPlugin;
use bevy::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(CustomAssetsPlugin)
        .add_startup_system(setup.system())
        .add_startup_system(load_and_play_audio.system())
        .add_system(animate_sprite_system.system())
        .run();
}

fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        }
    }
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // let background_handle = asset_server.load("data/RES/MAINMENU.PCX");

    // let texture_handle = asset_server.load("data/ANI/BWALK3.ANI");
    // let texture_handle = asset_server.load("data/ANI/KFACE.ANI");
    let texture_handle = asset_server.load("data/ANI/HEADWIPE.ANI");
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(73.0, 73.0), 1, 210);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn(Camera2dBundle::default())
        // .spawn(SpriteBundle {
        //     material: materials.add(background_handle.into()),
        //     ..Default::default()
        // })
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..Default::default()
        })
        .with(Timer::from_seconds(1. / 25., true));
}

fn load_and_play_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    // let music = asset_server.load("data/SOUND/MENU.RSS");
    // audio.play(music);
}
