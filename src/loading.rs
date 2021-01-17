use crate::asset_loaders::*;
use crate::state::*;
use bevy::prelude::*;

#[derive(Default)]
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .add_plugin(CustomAssetLoaders)
            .init_resource::<LoadingState>()            
            .on_state_enter(STAGE, AppState::Loading, setup_loading.system())
            .on_state_update(STAGE, AppState::Loading, loading.system())
            .on_state_exit(STAGE, AppState::Loading, cleanup_loading.system());
    }
}

#[derive(Default)]
struct LoadingState {
    animation_list_handle: Handle<AnimationList>,
}

struct LoadingData {
    text_entity: Entity,
}

struct LoadingText;

fn setup_loading(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut loading_state: ResMut<LoadingState>,
) {
    loading_state.animation_list_handle = asset_server.load("data/ANI/MASTER.ALI");

    commands
        .spawn(CameraUiBundle::default())
        .spawn(TextBundle {
            text: Text {
                value: "Loading...".to_string(),
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                style: TextStyle {
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..Default::default()
                },
            },
            ..Default::default()
        })
        .with(LoadingText);
    commands.insert_resource(LoadingData {
        text_entity: commands.current_entity().unwrap(),
    });
}

fn loading(
    commands: &mut Commands,
    mut state: ResMut<State<AppState>>,
    mut loading_state: ResMut<LoadingState>,
    animation_list_assets: ResMut<Assets<AnimationList>>,
    mut query: Query<&mut Text, With<LoadingText>>,
) {
    let list = animation_list_assets.get(&loading_state.animation_list_handle);
    if list.is_none() {
        return;
    }

    let list = list.unwrap();
    println!("{:?}", list);

    for mut text in query.iter_mut() {
        text.value = "Loaded animation list".to_owned();
    }

    // state.set_next(AppState::Menu).unwrap();
}

fn cleanup_loading(commands: &mut Commands, loading_data: Res<LoadingData>) {
    commands.despawn_recursive(loading_data.text_entity);
}
