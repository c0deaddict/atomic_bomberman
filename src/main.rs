use atomic_bomberman::asset_loaders::{AnimationBundle, CustomAssetLoaders};
use bevy::prelude::*;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(CustomAssetLoaders)
        .init_resource::<State>()
        .add_startup_system(setup.system())
        .add_startup_system(load_and_play_audio.system())
        .add_system(animate_sprite_system.system())
        .add_system(create_sprite_on_load.system())
        .run();
}

#[derive(Default)]
struct State {
    handle: Handle<AnimationBundle>,
    constructed: bool,
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
    mut state: ResMut<State>,
) {
    // TODO: load all ANI's
    // https://stackoverflow.com/questions/65330265/how-can-i-load-all-the-audio-files-inside-a-folder-using-bevy
    // let bundles: Vec<HandleUntyped> = asset_server.load_folder("data/ANI").unwrap();

    let background_handle = asset_server.load("data/RES/MAINMENU.PCX");

    // problematic:
    // XPLODE17.ANI
    // CORNERS6.ANI
    // CONVEYOR.ANI (different dims in SEQ)
    state.handle = asset_server.load("data/ANI/HEADWIPE.ANI");
    // let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(110.0, 110.0), 1, 15);
    // let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn(Camera2dBundle::default())
        .spawn(SpriteBundle {
            material: materials.add(background_handle.into()),
            transform: Transform::from_scale(Vec3::splat(2.0)),
            ..Default::default()
        });
}

fn load_and_play_audio(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    // let music = asset_server.load("data/SOUND/MENU.RSS");
    // audio.play(music);
}

fn create_sprite_on_load(
    commands: &mut Commands,
    mut state: ResMut<State>,
    animation_assets: ResMut<Assets<AnimationBundle>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut textures: ResMut<Assets<Texture>>,    
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let bundle = animation_assets.get(&state.handle);
    if state.constructed || bundle.is_none() {
        return;
    }

    let bundle = bundle.unwrap();

    // TODO: could also spawn textureAtlas from the animation loader
    let texture_atlas = TextureAtlas::from_grid(
        bundle.texture.clone(),
        Vec2::new(bundle.tile_width as f32, bundle.tile_height as f32),
        1,
        bundle.tile_count,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..Default::default()
        })
        .with(Timer::from_seconds(1. / 25., true));

    state.constructed = true;
}
