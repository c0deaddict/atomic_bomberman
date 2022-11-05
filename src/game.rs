use crate::{
    animation::*,
    asset_loaders::*,
    bomb::*,
    grid::{Direction, *},
    player::*,
    state::*,
};
use bevy::prelude::*;

#[derive(Default)]
pub struct GamePlugin;

#[derive(Component)]
pub struct PlayerPosText;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AnimatedSpritePlugin)
            .add_plugin(GridPlugin)
            .add_plugin(PlayerPlugin)
            .add_plugin(BombPlugin)
            .add_system_set(SystemSet::on_enter(AppState::Game).with_system(setup))
            .add_system_set(
                SystemSet::on_update(AppState::Game).with_system(player_pos_text),
            )
            .add_system_set(SystemSet::on_exit(AppState::Game).with_system(cleanup));
    }
}

fn setup(
    mut commands: Commands,
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
    commands.spawn_bundle(SpriteBundle {
        texture: background_handle,
        transform: Transform::from_translation(Vec3::new(320., -240., 10.)),
        ..Default::default()
    });

    // let music = asset_server.load("data/SOUND/MENU.RSS");
    // audio.play(music);

    let player_direction = PlayerDirection {
        direction: Direction::North,
        walking: false,
    };

    let animation = named_assets
        .animations
        .get(&player_direction.animation())
        .unwrap();

    let start_pos = Position { x: 7, y: 5 };
    commands
        .spawn_bundle((
            Transform::from_translation(Vec3::from((start_pos.into(), 30.0))),
            GlobalTransform::default(),
        ))
        .insert(Player)
        .insert(player_direction)
        .insert(KeyboardMovement::default())
        .insert(start_pos)
        .insert(PlayerSpeed{ timer: Timer::from_seconds(1. / 60., true)})
        .with_children(|parent| {
            let shadow = named_assets.animations.get("shadow").unwrap();

            parent.spawn_bundle(AnimatedSpriteBundle::new(
                shadow.clone(),
                &animation_assets,
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ));

            parent.spawn_bundle(AnimatedSpriteBundle::new(
                animation.clone(),
                &animation_assets,
                Transform::from_translation(Vec3::new(0.0, 35.0, 0.0)),
            ));
        });

    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(
                "Pos".to_string(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 16.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    vertical: VerticalAlign::Bottom,
                    horizontal: HorizontalAlign::Right,
                },
            ),
            transform: Transform::from_translation(Vec3::new(320., -5., 100.)),
            ..Default::default()
        })
        .insert(PlayerPosText);

    let scheme = scheme_assets
        .get(named_assets.schemes.get("X MARKS THE SPOT (10)").unwrap())
        .unwrap();

    let brick = named_assets.animations.get("tile 0 brick").unwrap();
    let solid = named_assets.animations.get("tile 0 solid").unwrap();
    let blank = named_assets.animations.get("tile 0 blank").unwrap();

    for x in 0..15 {
        for y in 0..11 {
            let cell = scheme.grid[y][x];
            let animation = match cell {
                Cell::Solid => solid,
                Cell::Brick => brick,
                Cell::Blank => blank,
            };

            commands
                .spawn_bundle(AnimatedSpriteBundle::new(
                    animation.clone(),
                    &animation_assets,
                    Default::default(),
                ))
                .insert(Position {
                    x: x as i32,
                    y: y as i32,
                })
                .insert(Offset(Vec3::new(0.0, 0.0, 20.0)))
                .insert(SnapToGrid)
                .insert(cell);
        }
    }
}

fn cleanup() {
    info!("cleanup");
}

fn player_pos_text(
    mut queries: ParamSet<(
        Query<&mut Text, With<PlayerPosText>>,
        Query<&Position, With<Player>>,
    )>,
) {
    let mut p = Position { x: 0, y: 0 };
    for pos in queries.p1().iter() {
        p = *pos
    }

    for mut text in queries.p0().iter_mut() {
        text.sections[0].value = format!("{}x{}", p.x, p.y);
    }
}
