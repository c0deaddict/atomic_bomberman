use atomic_bomberman::{animated_sprite::*, asset_loaders::*};
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use std::cmp::PartialEq;
use std::env;
use std::fmt::Debug;

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(CustomAssetLoaders)
        .add_plugin(AnimatedSpritePlugin)
        .add_resource(TestTimer(Timer::from_seconds(3.0, true)))
        .init_resource::<State>()
        .add_startup_system(setup.system())
        .add_startup_system(load_and_play_audio.system())
        .add_system(create_sprite_on_load.system())
        .add_system(player_movement.system())
        .add_system(change_sprite.system())
        .run();
}

// TODO: menu and asset loading screen
// https://github.com/bevyengine/bevy/blob/master/examples/ecs/state.rs

// TODO: change the sprite of an entity:
// https://github.com/bevyengine/bevy/issues/498

#[derive(Debug, PartialEq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn next(&self) -> Direction {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }

    fn animation_index(&self) -> usize {
        match self {
            Direction::North => 0,
            Direction::East => 3,
            Direction::South => 1,
            Direction::West => 2,
        }
    }
}

struct Player {}

#[derive(Debug)]
struct PlayerDirection(Direction);

struct TestTimer(Timer);

#[derive(Default)]
struct State {
    handle: Handle<AnimationBundle>,
    loaded: bool,
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut state: ResMut<State>,
) {
    // color is always GREEN how to change to other colors??
    // is that what the PAL header is for??

    // player: STAND.ANI (four perspectives)
    //   WALK.ANI for walking

    //  in "data/ANI/MFLAME.ANI": CIMG 16bpp expected no palette)

    // TODO: load text file: data/ANI/MASTER.ALI
    // and include all ANI's that are in that file.

    // https://stackoverflow.com/questions/65330265/how-can-i-load-all-the-audio-files-inside-a-folder-using-bevy
    // let bundles: Vec<HandleUntyped> = asset_server.load_folder("data/ANI").unwrap();

    // let texture_handle = asset_server.load("data/RES/MAINMENU.PCX");
    // let texture_handle = asset_server.load("data/COLOR.PAL");

    let args: Vec<String> = env::args().collect();

    // problematic:
    // XPLODE17.ANI
    // CORNER6.ANI
    // CONVEYOR.ANI (different dims in SEQ)
    // data/ANI/CLASSICS.ANI
    let filename = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or("data/ANI/WALK.ANI");
    state.handle = asset_server.load(filename);
    // state.handle = asset_server.load("data/ANI/CLASSICS.ANI");
    // let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(110.0, 110.0), 1, 15);
    // let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands
        .spawn(Camera2dBundle::default())
        // .spawn(UiCameraComponents::default())        
        // .spawn(SpriteBundle {
        //     material: materials.add(texture_handle.into()),
        //     transform: Transform::from_scale(Vec3::splat(8.0)),
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

    // for (i, animation) in bundle.animations.iter().cycle().take(12 * 8).enumerate() {
    //     let x = -(80. * 6.) + (i % 12) as f32 * 80.0;
    //     let y = -(80. * 4.) + (i / 12) as f32 * 80.0;
    //     commands
    //         // .spawn(Player::default())
    //         .spawn(SpriteSheetBundle {
    //             texture_atlas: bundle.texture_atlas.clone(),
    //             transform: Transform::from_scale(Vec3::splat(2.0))
    //                 .mul_transform(Transform::from_translation(Vec3::new(x, y, 0.0))),
    //             ..Default::default()
    //         })
    //         .with(AnimateSprite {
    //             animation: animation.clone(),
    //             index: 0, // i % animation.frames.len(),
    //         })
    //         .with(Timer::from_seconds(1. / 25., true));
    // }

    let animation = &bundle.animations[0];

    commands
        .spawn((
            Transform::from_translation(Vec3::new(-50.0, 0.0, 0.0)),
            GlobalTransform::default(),
            Player {},
            PlayerDirection(Direction::North),
        ))
        .with_children(|parent| {
            AnimatedSprite::spawn_child(parent, &bundle, animation);
        });

    state.loaded = true;
}

// NOTE: systems with Changed<> must be added AFTER the system that triggers the change.
fn change_sprite(
    commands: &mut Commands,
    time: Res<Time>,
    animation_assets: ResMut<Assets<AnimationBundle>>,
    mut state: Res<State>,
    mut timer: ResMut<TestTimer>,
    mut query: Query<(Entity, &Children, &PlayerDirection), Changed<PlayerDirection>>,
) {
    let bundle = animation_assets.get(&state.handle);
    if bundle.is_none() {
        return;
    }

    let bundle = bundle.unwrap();

    for (entity, children, direction) in query.iter_mut() {
        println!("direction changed to: {:?}", direction.0);

        let child = children.first().copied().unwrap();
        commands.despawn_recursive(child);

        let animation = &bundle.animations[direction.0.animation_index()];
        println!("animation {}", animation.name);

        // only add timer if frames.length > 1
        let child = AnimatedSprite::spawn(commands, &bundle, animation)
            .current_entity()
            .unwrap();

        commands.push_children(entity, &[child]);
    }
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut PlayerDirection)>,
) {
    for (mut transform, mut direction) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= 2.;
            if direction.0 != Direction::West {
                direction.0 = Direction::West;
            }
        }
        if keyboard_input.pressed(KeyCode::D) {
            transform.translation.x += 2.;
            if direction.0 != Direction::East {
                direction.0 = Direction::East;
            }
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.translation.y -= 2.;
            if direction.0 != Direction::South {
                direction.0 = Direction::South;
            }
        }
        if keyboard_input.pressed(KeyCode::W) {
            transform.translation.y += 2.;
            if direction.0 != Direction::North {
                direction.0 = Direction::North;
            }
        }
    }
}

// https://github.com/bevyengine/bevy/pull/273

// fn counter_system(diagnostics: Res<Diagnostics>, mut text: Mut<Text>) {
//     if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
//         if let Some(average) = fps.average() {
//             text.value = format!("FPS: {:.2}", average);
//         }
//     };
// }
