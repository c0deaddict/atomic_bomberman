use atomic_bomberman::asset_loaders::*;
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

struct AnimateSprite {
    animation: Animation,
    index: usize,
}

#[derive(Default)]
struct State {
    handle: Handle<AnimationBundle>,
    loaded: bool,
}

fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut Timer,
        &mut TextureAtlasSprite,
        &mut AnimateSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, mut animate_sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            animate_sprite.index =
                (animate_sprite.index + 1) % animate_sprite.animation.frames.len();
            sprite.index = animate_sprite.animation.frames[animate_sprite.index].index as u32;
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

    // let texture_handle = asset_server.load("data/RES/MAINMENU.PCX");

    // problematic:
    // XPLODE17.ANI
    // CORNER6.ANI
    // CONVEYOR.ANI (different dims in SEQ)
    // data/ANI/CLASSICS.ANI
    state.handle = asset_server.load("data/ANI/HEADWIPE.ANI");
    // state.handle = asset_server.load("data/ANI/CLASSICS.ANI");
    // let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(110.0, 110.0), 1, 15);
    // let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn(Camera2dBundle::default())
        // .spawn(SpriteBundle {
        //     material: materials.add(texture_handle.into()),
        //     transform: Transform::from_scale(Vec3::splat(2.0)),
        //     ..Default::default()
        // })
        ;
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
    if state.loaded || bundle.is_none() {
        return;
    }

    let bundle = bundle.unwrap();

    for (i, animation) in bundle.animations.iter().cycle().take(12*8).enumerate() {
        let x = -(80.*6.) + (i % 12) as f32 * 80.0;
        let y = -(80.*4.) + (i / 12) as f32 * 80.0;
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: bundle.texture_atlas.clone(),
                transform: Transform::from_scale(Vec3::splat(2.0))
                    .mul_transform(Transform::from_translation(Vec3::new(x, y, 0.0))),
                ..Default::default()
            })
            .with(AnimateSprite {
                animation: animation.clone(),
                index: i % animation.frames.len(),
            })
            .with(Timer::from_seconds(1. / 25., true));
    }

    state.loaded = true;
}
