use crate::{animated_sprite::*, asset_loaders::*, state::*};
use bevy::prelude::*;
use std::cmp::PartialEq;
use std::fmt::Debug;

#[derive(Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(AnimatedSpritePlugin)
            .on_state_enter(STAGE, AppState::Game, setup.system())
            .on_state_update(STAGE, AppState::Game, player_movement.system())
            .on_state_update(STAGE, AppState::Game, change_sprite.system())
            .on_state_exit(STAGE, AppState::Game, cleanup.system());
    }
}

#[derive(Debug, PartialEq)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn animation(&self) -> &str {
        match self {
            Direction::North => "walk north",
            Direction::East => "walk east",
            Direction::South => "walk south",
            Direction::West => "walk west",
        }
    }
}

struct Player {}

#[derive(Debug)]
struct PlayerDirection(Direction);

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    // audio: Res<Audio>,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // player: STAND.ANI (four perspectives)
    //   WALK.ANI for walking

    let background_handle = asset_server.load("data/RES/FIELD0.PCX");
    commands.spawn(SpriteBundle {
        material: materials.add(background_handle.into()),
        transform: Transform::from_translation(Vec3::new(320., -240., 0.)),
        ..Default::default()
    });

    // let music = asset_server.load("data/SOUND/MENU.RSS");
    // audio.play(music);

    let direction = Direction::North;
    let animation = named_assets.animations.get(direction.animation()).unwrap();

    commands
        .spawn((
            Transform::from_translation(Vec3::new(320., -240., 0.0)),
            GlobalTransform::default(),
            Player {},
            PlayerDirection(direction),
        ))
        .with_children(|parent| {
            AnimatedSprite::spawn_child(parent, animation.clone(), &animation_assets);
        });
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

// NOTE: systems with Changed<> must be added AFTER the system that triggers the change.
fn change_sprite(
    commands: &mut Commands,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    mut query: Query<(Entity, &Children, &PlayerDirection), Changed<PlayerDirection>>,
) {
    for (entity, children, direction) in query.iter_mut() {
        println!("direction changed to: {:?}", direction.0);

        if let Some(child) = children.first().copied() {
            commands.despawn_recursive(child);
        }

        let animation = named_assets
            .animations
            .get(direction.0.animation())
            .unwrap();

        let child = AnimatedSprite::spawn(commands, animation.clone(), &animation_assets)
            .current_entity()
            .unwrap();

        commands.push_children(entity, &[child]);
    }
}

fn cleanup(commands: &mut Commands) {
    // commands.despawn_recursive(loading_state.text_entity);
}
