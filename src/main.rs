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
        .add_resource(State::new(AppState::Loading))
        .add_resource(bevy::log::LogSettings {
            level: bevy::log::Level::DEBUG,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_stage_after(stage::UPDATE, STAGE, StateStage::<AppState>::default())
        .add_plugin(WindowPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin)
        .add_plugin(LoadingPlugin)
        .add_plugin(GamePlugin)
        .add_startup_system(setup.system())
        .add_system(fps_counter_system.system())
        .run();
}

struct FpsText;

fn setup(commands: &mut Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(Text2dBundle {
            text: Text {
                value: "FPS".to_string(),
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                style: TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Bottom,
                        horizontal: HorizontalAlign::Right,
                    },
                },
            },
            transform: Transform::from_translation(Vec3::new(2., -2., 3.)),
            ..Default::default()
        })
        .with(FpsText);
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
                text.value = format!("FPS: {:.2} #ent {}", average, entities.iter().count());
            }
        }
    };
}
