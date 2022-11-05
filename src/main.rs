use atomic_bomberman::{game::GamePlugin, loading::LoadingPlugin, state::*, window::WindowPlugin};
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

// TODO: use https://bevy-cheatbook.github.io/features/audio.html
// TODO: https://bevy-cheatbook.github.io/features/fixed-timestep.html

// TODO: Generate animations based on color.pal (other asset)
// Could listen to AssetEvent to wait for palette resource to be loaded.
// How to pass data to AssetLoaders?
//
// https://github.com/bevyengine/bevy/pull/1665
// https://github.com/bevyengine/bevy/pull/1393
// https://github.com/bevyengine/bevy/discussions/1307

// COLOR.PAL: (= palette)
// 256 RGB colors (768 bytes).
// 2^15 indices (1 byte) into color table (32768 bytes)
//
// *.RMP (color remap table):
// 256 byte to byte mapping.
//
// v = 2^15 color from animation
// remapped = palette[3 * rmp[palette[768 + v]]] # 3 bytes starting at this addr.
//
// Player colors:
// 1 = white
// 2 = black
// 3 = red
// 4 = blue
// 5 = green
// 6 = yellow
// 7 = turqoise
// 8 = pink
// 9 = orange
// 10 = purple

fn main() {
    App::new()
        .add_state(AppState::Loading)
        .insert_resource(bevy::log::LogSettings {
            level: bevy::log::Level::INFO,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(WindowPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(LoadingPlugin)
        .add_plugin(GamePlugin)
        .add_startup_system(setup)
        .add_system(fps_counter_system)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Component)]
struct FpsText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(Text2dBundle {
            text: Text::with_section(
                "FPS".to_string(),
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
            transform: Transform::from_translation(Vec3::new(10., -5., 100.)),
            ..Default::default()
        })
        .insert(FpsText);
}

// https://github.com/bevyengine/bevy/pull/273

fn fps_counter_system(
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<FpsText>>,
    entities: Query<Entity>,
) {
    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            for mut text in query.iter_mut() {
                text.sections[0].value =
                    format!("FPS: {:.2} #ent {}", average, entities.iter().count());
            }
        }
    };
}
