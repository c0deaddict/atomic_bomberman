use crate::{animated_sprite::*, asset_loaders::*, state::*};
use bevy::prelude::*;
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;

#[derive(Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(AnimatedSpritePlugin)
            .on_state_enter(STAGE, AppState::Game, setup.system())
            .on_state_update(STAGE, AppState::Game, player_movement.system())
            .on_state_update(STAGE, AppState::Game, place_bomb.system())
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

#[derive(Clone, PartialEq, Eq, Debug)]
struct Pos {
    x: u8,
    y: u8,
}

struct Bombs(Vec<Pos>);

#[derive(Debug)]
struct PlayerDirection(Direction);

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    // audio: Res<Audio>,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    scheme_assets: Res<Assets<Scheme>>,
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
        ))
        .with(Player {})
        .with(PlayerDirection(direction))
        .with(Pos { x: 0, y: 0 })
        .with(Bombs(vec![]))
        .with_children(|parent| {
            AnimatedSprite::spawn_child(parent, animation.clone(), &animation_assets);
        });

    let scheme = scheme_assets
        .get(named_assets.schemes.get("X MARKS THE SPOT (10)").unwrap())
        .unwrap();

    let brick = named_assets.animations.get("tile 0 brick").unwrap();
    let solid = named_assets.animations.get("tile 0 solid").unwrap();
    let blank = named_assets.animations.get("tile 0 blank").unwrap();

    for x in 0..15 {
        for y in 0..11 {
            let tx = 20.0 + x as f32 * 40.0;
            let ty = -64.0 - y as f32 * 36.0;

            let animation = match scheme.grid[y][x] {
                Cell::Solid => solid,
                Cell::Brick => brick,
                Cell::Blank => blank,
            };

            commands
                .spawn((
                    Transform::from_translation(Vec3::new(tx + 20.0, ty - 18.0, 0.0)),
                    GlobalTransform::default(),
                ))
                .with_children(|parent| {
                    AnimatedSprite::spawn_child(parent, animation.clone(), &animation_assets);
                });
        }
    }
}

fn update_pos_from_transform(transform: &Transform, pos: &mut Pos) {
    // player is 110x110
    pos.x = ((transform.translation.x - 20.) / 40.) as u8;
    pos.y = ((transform.translation.y - 32.0 - -64.0) / -36.) as u8;
}

fn player_movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut PlayerDirection, &mut Pos)>,
) {
    for (mut transform, mut direction, mut pos) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= 2.;
            update_pos_from_transform(&transform, &mut pos);
            if direction.0 != Direction::West {
                direction.0 = Direction::West;
            }
        }
        if keyboard_input.pressed(KeyCode::D) {
            transform.translation.x += 2.;
            update_pos_from_transform(&transform, &mut pos);
            if direction.0 != Direction::East {
                direction.0 = Direction::East;
            }
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.translation.y -= 2.;
            update_pos_from_transform(&transform, &mut pos);
            if direction.0 != Direction::South {
                direction.0 = Direction::South;
            }
        }
        if keyboard_input.pressed(KeyCode::W) {
            transform.translation.y += 2.;
            update_pos_from_transform(&transform, &mut pos);
            if direction.0 != Direction::North {
                direction.0 = Direction::North;
            }
        }
    }
}

fn place_bomb(
    commands: &mut Commands,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Pos, &mut Bombs)>,
    audio: Res<Audio>,
) {
    for (pos, mut bombs) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Space) {
            if !bombs.0.contains(&pos) {
                let animation = named_assets.animations.get("bomb regular green").unwrap();
                let x = 20.0 + pos.x as f32 * 40.0;
                let y = -64.0 + pos.y as f32 * -36.0;
                commands
                    .spawn((
                        Transform::from_translation(Vec3::new(x + 24., y - 20., 0.0)),
                        GlobalTransform::default(),
                    ))
                    .with_children(|parent| {
                        AnimatedSprite::spawn_child(parent, animation.clone(), &animation_assets);
                    });

                let handle = named_assets.sounds.get("bmdrop2").unwrap();
                audio.play(handle.clone());

                bombs.0.push(pos.clone());
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

fn cleanup() {
    info!("cleanup");
}
