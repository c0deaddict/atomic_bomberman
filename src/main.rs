use atomic_bomberman::{game::GamePlugin, loading::LoadingPlugin, state::*, window::WindowPlugin};
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

// TODO: figure out how to remap colors for player and flame animations.
// for each color there is RMP file in the root bomberman directory.
// i guess the COLOR.PAL file also has to do something with it,
// or maybe the palette header in the ANI files?

fn main() {
    App::build()
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
        .add_startup_system(setup.system())
        .add_system(fps_counter_system.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}

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
