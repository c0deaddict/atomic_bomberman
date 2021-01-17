use atomic_bomberman::{
    state::*,
    loading::LoadingPlugin,
};
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_resource(State::new(AppState::Loading))
        .add_stage_after(stage::UPDATE, STAGE, StateStage::<AppState>::default())
        .add_plugin(LoadingPlugin)
        .on_state_enter(STAGE, AppState::Menu, setup_menu.system())
        .on_state_update(STAGE, AppState::Menu, menu.system())
        .on_state_exit(STAGE, AppState::Menu, cleanup_menu.system())
        .run();
}

// TODO: menu and asset loading screen
// https://github.com/bevyengine/bevy/blob/master/examples/ecs/state.rs

// TODO: custom coordinate system: 640x480

// TODO: figure out how to remap colors for player and flame animations.
// for each color there is RMP file in the root bomberman directory.
// i guess the COLOR.PAL file also has to do something with it,
// or maybe the palette header in the ANI files?

// TODO: load/hardcode? SOUNDLST.RES for a list of all sounds

fn setup_menu(commands: &mut Commands) {
    println!("In state menu");
}

fn menu(commands: &mut Commands) {}

fn cleanup_menu(commands: &mut Commands) {}
