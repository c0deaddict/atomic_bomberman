use crate::asset_loaders::*;
use crate::state::*;

use bevy::prelude::*;
use bevy::window::WindowResized;
use std::collections::HashMap;

#[derive(Default)]
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(CustomAssetLoaders)
            .init_resource::<LoadingState>()
            .init_resource::<Resources>()
            .add_system(window_resize.system())
            .on_state_enter(STAGE, AppState::Loading, setup_loading.system())
            .on_state_update(STAGE, AppState::Loading, load_animations.system())
            .on_state_update(STAGE, AppState::Loading, load_sounds.system())
            .on_state_update(STAGE, AppState::Loading, loading_progress.system())
            .on_state_exit(STAGE, AppState::Loading, cleanup_loading.system());
    }
}

/// All animations listed in DATA/ANI/MASTER.ALI
const ANIMATION_LIST: &'static [&str] = &[
    "ALIENS1.ANI",
    "BOMBS.ANI",
    "BWALK1.ANI",
    "BWALK2.ANI",
    "BWALK3.ANI",
    "BWALK4.ANI",
    "CONVEYOR.ANI",
    "CORNER0.ANI",
    "CORNER1.ANI",
    "CORNER2.ANI",
    "CORNER3.ANI",
    "CORNER4.ANI",
    "CORNER5.ANI",
    "CORNER6.ANI",
    "CORNER7.ANI",
    "DUDS.ANI",
    "EDIT.ANI",
    "EXTRAS.ANI",
    "HURRY.ANI",
    "KFACE.ANI",
    "KFONT.ANI",
    "KICK.ANI",
    "MFLAME.ANI",
    "MISC.ANI",
    "POWERS.ANI",
    "PUNBOMB1.ANI",
    "PUNBOMB2.ANI",
    "PUNBOMB3.ANI",
    "PUNBOMB4.ANI",
    "PUP1.ANI",
    "PUP2.ANI",
    "PUP3.ANI",
    "PUP4.ANI",
    "SHADOW.ANI",
    "STAND.ANI",
    "TILES0.ANI",
    "TILES1.ANI",
    "TILES10.ANI",
    "TILES2.ANI",
    "TILES3.ANI",
    "TILES4.ANI",
    "TILES5.ANI",
    "TILES6.ANI",
    "TILES7.ANI",
    "TILES8.ANI",
    "TILES9.ANI",
    "TRIGANIM.ANI",
    "WALK.ANI",
    "XBRICK0.ANI",
    "XBRICK1.ANI",
    "XBRICK10.ANI",
    "XBRICK2.ANI",
    "XBRICK3.ANI",
    "XBRICK4.ANI",
    "XBRICK5.ANI",
    "XBRICK6.ANI",
    "XBRICK7.ANI",
    "XBRICK8.ANI",
    "XBRICK9.ANI",
    "XPLODE1.ANI",
    "XPLODE10.ANI",
    "XPLODE11.ANI",
    "XPLODE12.ANI",
    "XPLODE13.ANI",
    "XPLODE14.ANI",
    "XPLODE15.ANI",
    "XPLODE16.ANI",
    "XPLODE17.ANI",
    "XPLODE2.ANI",
    "XPLODE3.ANI",
    "XPLODE4.ANI",
    "XPLODE5.ANI",
    "XPLODE6.ANI",
    "XPLODE7.ANI",
    "XPLODE8.ANI",
    "XPLODE9.ANI",
];

/// All sounds listed in DATA/RES/SOUNDLST.RES
const SOUND_LIST: &'static [&str] = &[
    "1006.RSS",
    "1017.RSS",
    "1028.RSS",
    "1036.RSS",
    "1041.RSS",
    "1045.RSS",
    "1049.RSS",
    "1055.RSS",
    "1059.RSS",
    "1062.RSS",
    "1074.RSS",
    "2MUCPAIN.RSS",
    "ALLRITE.RSS",
    "BACKUP.RSS",
    "BACKUP1.RSS",
    "BMBSTOP1.RSS",
    "BMBSTOP2.RSS",
    "BMBTHRW1.RSS",
    "BMBTHRW3.RSS",
    "BMBTHRW4.RSS",
    "BMBTHRW5.RSS",
    "BMDROP1.RSS",
    "BMDROP2.RSS",
    "BMDROP3.RSS",
    "BOMBBOUN.RSS",
    "BOMBHIT1.RSS",
    "BOMBHIT2.RSS",
    "BOMBHIT3.RSS",
    "BOMBHIT4.RSS",
    "BOMBSTOP.RSS",
    "BOMB_01.RSS",
    "BOMB_02.RSS",
    "BOMB_04.RSS",
    "BOMB_04B.RSS",
    "BOMB_05.RSS",
    "BOMB_06.RSS",
    "BOMB_07.RSS",
    "BOMB_07B.RSS",
    "BOMB_09.RSS",
    "BOMB_11.RSS",
    "BOMB_12.RSS",
    "BOMB_12B.RSS",
    "BOMB_17.RSS",
    "BOMB_19.RSS",
    "BOMB_24.RSS",
    "BRINGON.RSS",
    "BURNEDUP.RSS",
    "CALLDAD.RSS",
    "CLEAR.RSS",
    "CLIKPLAT.RSS",
    "COMEGET.RSS",
    "COOLPOP.RSS",
    "CRIBROWN.RSS",
    "CUL8R.RSS",
    "DABOMB.RSS",
    "DESTU1.RSS",
    "DESTU2.RSS",
    "DIE1.RSS",
    "dis2a.rss",
    "DRABOM.RSS",
    "DRAW.RSS",
    "EATDUST.RSS",
    "ELAUGH1.RSS",
    "ENRT1.RSS",
    "ENRT2.RSS",
    "EWTAHH.RSS",
    "EXPL6.RSS",
    "EXPLO1.RSS",
    "EXPLODE2.RSS",
    "EXPLODE3.RSS",
    "EXPLODE4.RSS",
    "FEELPOWR.RSS",
    "FILLCRAK.RSS",
    "FIREINH.RSS",
    "GEN8A.RSS",
    "GET1.RSS",
    "GET2.RSS",
    "GOTAHURT.RSS",
    "GOTCHA.RSS",
    "GRAB1.RSS",
    "GRAB2.RSS",
    "GUMP1.RSS",
    "HITSTRID.RSS",
    "HURRY.RSS",
    "HURYTUF.RSS",
    "INZONE.RSS",
    "KBOMB1.RSS",
    "KBOMB2.RSS",
    "KICKER10.RSS",
    "KICKER3.RSS",
    "LATER.RSS",
    "LETTER1.RSS",
    "LETTER2.RSS",
    "LITEMUP.RSS",
    "LOOKOUT.RSS",
    "LOSE.RSS",
    "LOVNIT1.RSS",
    "LUVYAP.RSS",
    "MENU.RSS",
    "MENUEXIT.RSS",
    "MOLASES.RSS",
    "NETWORK.RSS",
    "OHHHH.RSS",
    "OHNO1.RSS",
    "OHYEAH.RSS",
    "PACKIN.RSS",
    "PGAD12B.RSS",
    "PLAYPROP.RSS",
    "POOPS1.RSS",
    "POOPS2.RSS",
    "POOPS3.RSS",
    "POOPS4.RSS",
    "POOPS5.RSS",
    "PROUD.RSS",
    "QUITGAME.RSS",
    "ROASTED.RSS",
    "RUNAWAY1.RSS",
    "RUNAWAY2.RSS",
    "SCHWING.RSS",
    "SCREAM1.RSS",
    "SERENDIP.RSS",
    "SHAKEBOO.RSS",
    "SMELSMOK.RSS",
    "SQRDROP2.RSS",
    "SQRDROP4.RSS",
    "SQRDROP5.RSS",
    "SQRDROP6.RSS",
    "SQRDROP7.RSS",
    "SQRDROP8.RSS",
    "STUPIDIO.RSS",
    "SUCKITDN.RSS",
    "TASTPAI2.RSS",
    "TASTPAIN.RSS",
    "THEMAN.RSS",
    "TOEASY.RSS",
    "TRAMPO.RSS",
    "WANTMNKY.RSS",
    "WARP1.RSS",
    "WHATRUSH.RSS",
    "WHODAD.RSS",
    "WIN.RSS",
    "WLKTAL1.RSS",
    "WLKTAL2.RSS",
    "WOOHOO1.RSS",
    "YEEHAW.RSS",
    "YOUBLOW.RSS",
    "YOUWIN1.RSS",
    "ZEN1.RSS",
    "ZEN2.RSS",
    "disease1.rss",
    "disease2.rss",
    "disease3.rss",
];

#[derive(Default)]
struct LoadingState {
    animations_loading: Vec<Handle<AnimationBundle>>,
    sounds_loading: HashMap<String, Handle<AudioSource>>,
    total_assets: usize,
}

#[derive(Default)]
struct Resources {
    animations: HashMap<String, Animation>,
    sounds: HashMap<String, Handle<AudioSource>>,
}

struct LoadingData {
    text_entity: Entity,
}

struct LoadingText;
struct WindowTransform;

fn setup_loading(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut loading_state: ResMut<LoadingState>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // TODO: show image like GLUE2.PCX as background?
    // TODO: which font is used in the original game?
    let background_handle = asset_server.load("data/RES/GLUE2.PCX");

    commands
        .spawn(Camera2dBundle::default())
        .spawn((
            Transform::from_scale(Vec3::splat(2.0))
                .mul_transform(Transform::from_translation(Vec3::new(0., 0., 0.))),
            GlobalTransform::default(),
        ))
        .with(WindowTransform)
        .with_children(|parent| {
            parent
                .spawn(SpriteBundle {
                    material: materials.add(background_handle.into()),
                    transform: Transform::from_translation(Vec3::new(320., -240., 0.)),
                    ..Default::default()
                })
                .spawn(Text2dBundle {
                    text: Text {
                        value: "Loading...".to_string(),
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        style: TextStyle {
                            font_size: 40.0,
                            color: Color::WHITE,
                            alignment: TextAlignment {
                                vertical: VerticalAlign::Center,
                                horizontal: HorizontalAlign::Center,
                            },
                        },
                    },
                    transform: Transform::from_translation(Vec3::new(320.0, -240.0, 0.)),
                    ..Default::default()
                })
                .with(LoadingText);
        });

    commands.insert_resource(LoadingData {
        text_entity: commands.current_entity().unwrap(),
    });

    for filename in ANIMATION_LIST {
        let path = "data/ANI/".to_owned() + filename;
        loading_state
            .animations_loading
            .push(asset_server.load(path.as_str()));
        loading_state.total_assets += 1;
    }

    for filename in SOUND_LIST {
        let path = "data/SOUND/".to_owned() + filename;
        let filename = (*filename).to_owned();
        loading_state
            .sounds_loading
            .insert(filename, asset_server.load(path.as_str()));
        loading_state.total_assets += 1;
    }
}

fn load_animations(
    commands: &mut Commands,
    mut loading_state: ResMut<LoadingState>,
    mut resources: ResMut<Resources>,
    animation_bundle_assets: Res<Assets<AnimationBundle>>,
) {
    let done = loading_state
        .animations_loading
        .drain_filter(|handle| animation_bundle_assets.contains(handle.clone()));

    for handle in done {
        let bundle = animation_bundle_assets.get(handle).unwrap();
        for animation in bundle.animations.iter() {
            resources
                .animations
                .insert(animation.name.clone(), animation.clone());
        }
    }
}

fn load_sounds(
    commands: &mut Commands,
    mut loading_state: ResMut<LoadingState>,
    mut resources: ResMut<Resources>,
    sound_assets: Res<Assets<AudioSource>>,
) {
    let done = loading_state
        .sounds_loading
        .drain_filter(|_filename, handle| sound_assets.contains(handle.clone()));

    for (filename, handle) in done {
        resources.sounds.insert(filename, handle);
    }
}

fn loading_progress(
    commands: &mut Commands,
    mut state: ResMut<State<AppState>>,
    loading_state: ResMut<LoadingState>,
    mut query: Query<&mut Text, With<LoadingText>>,
) {
    let remaining = loading_state.animations_loading.len() + loading_state.sounds_loading.len();
    let assets_loaded = loading_state.total_assets - remaining;

    for mut text in query.iter_mut() {
        text.value = format!(
            "Loading {}/{} assets",
            assets_loaded, loading_state.total_assets
        );
    }

    if assets_loaded == loading_state.total_assets {
        // state.set_next(AppState::Menu).unwrap();
    }
}

fn window_resize(
    resize_event: Res<Events<WindowResized>>,
    mut query: Query<&mut Transform, With<WindowTransform>>,
) {
    let mut event_reader = resize_event.get_reader();
    for event in event_reader.iter(&resize_event) {
        for mut transform in query.iter_mut() {
            transform.scale.x = event.width / 640.;
            transform.scale.y = event.height / 480.;
            transform.translation.x = -event.width / 2.;
            transform.translation.y = event.height / 2.;
            println!("Info: {:?} {:?}", event, transform);
        }
    }
}

fn cleanup_loading(commands: &mut Commands, loading_data: Res<LoadingData>) {
    trace!("very noisy");
    debug!("helpful for debugging");
    info!("helpful information that is worth printing by default");
    warn!("something bad happened that isn't a failure, but thats worth calling out");
    error!("something failed");

    commands.despawn_recursive(loading_data.text_entity);
}
