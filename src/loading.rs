use crate::asset_loaders::*;
use crate::animation::*;
use crate::state::*;

use bevy::prelude::*;
use std::collections::HashMap;
use std::path::Path;

#[derive(Default)]
pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(CustomAssetLoaders)
            .init_resource::<AssetsLoading>()
            .init_resource::<NamedAssets>()
            .on_state_enter(STAGE, AppState::Loading, setup.system())
            .on_state_update(STAGE, AppState::Loading, load_animations.system())
            .on_state_update(STAGE, AppState::Loading, load_sounds.system())
            .on_state_update(STAGE, AppState::Loading, load_schemes.system())
            .on_state_update(STAGE, AppState::Loading, loading_progress.system())
            .on_state_exit(STAGE, AppState::Loading, cleanup.system());
    }
}

/// All animations listed in DATA/ANI/MASTER.ALI
const ANIMATION_LIST: &'static [&str] = &[
    // "ALIENS1.ANI",
    "BOMBS.ANI",
    // "BWALK1.ANI",
    // "BWALK2.ANI",
    // "BWALK3.ANI",
    // "BWALK4.ANI",
    // "CONVEYOR.ANI",
    // "CORNER0.ANI",
    // "CORNER1.ANI",
    // "CORNER2.ANI",
    // "CORNER3.ANI",
    // "CORNER4.ANI",
    // "CORNER5.ANI",
    // "CORNER6.ANI",
    // "CORNER7.ANI",
    // "DUDS.ANI",
    // "EDIT.ANI",
    // "EXTRAS.ANI",
    // "HURRY.ANI",
    // "KFACE.ANI",
    // "KFONT.ANI",
    // "KICK.ANI",
    // "MFLAME.ANI",
    // "MISC.ANI",
    // "POWERS.ANI",
    // "PUNBOMB1.ANI",
    // "PUNBOMB2.ANI",
    // "PUNBOMB3.ANI",
    // "PUNBOMB4.ANI",
    // "PUP1.ANI",
    // "PUP2.ANI",
    // "PUP3.ANI",
    // "PUP4.ANI",
    // "SHADOW.ANI",
    "STAND.ANI",
    "TILES0.ANI",
    // "TILES1.ANI",
    // "TILES10.ANI",
    // "TILES2.ANI",
    // "TILES3.ANI",
    // "TILES4.ANI",
    // "TILES5.ANI",
    // "TILES6.ANI",
    // "TILES7.ANI",
    // "TILES8.ANI",
    // "TILES9.ANI",
    // "TRIGANIM.ANI",
    "WALK.ANI",
    // "XBRICK0.ANI",
    // "XBRICK1.ANI",
    // "XBRICK10.ANI",
    // "XBRICK2.ANI",
    // "XBRICK3.ANI",
    // "XBRICK4.ANI",
    // "XBRICK5.ANI",
    // "XBRICK6.ANI",
    // "XBRICK7.ANI",
    // "XBRICK8.ANI",
    // "XBRICK9.ANI",
    // "XPLODE1.ANI",
    // "XPLODE10.ANI",
    // "XPLODE11.ANI",
    // "XPLODE12.ANI",
    // "XPLODE13.ANI",
    // "XPLODE14.ANI",
    // "XPLODE15.ANI",
    // "XPLODE16.ANI",
    // "XPLODE17.ANI",
    // "XPLODE2.ANI",
    // "XPLODE3.ANI",
    // "XPLODE4.ANI",
    // "XPLODE5.ANI",
    // "XPLODE6.ANI",
    // "XPLODE7.ANI",
    // "XPLODE8.ANI",
    // "XPLODE9.ANI",
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
struct AssetsLoading {
    animations: Vec<Handle<AnimationBundle>>,
    sounds: HashMap<String, Handle<AudioSource>>,
    schemes: Vec<Handle<Scheme>>,
    initial_count: usize,
}

struct LoadingText;

struct LoadingScreen {
    entity: Entity,
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    mut assets_loading: ResMut<AssetsLoading>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // TODO: which font is used in the original game?
    let background_handle = asset_server.load("data/RES/GLUE2.PCX");

    commands
        .spawn(SpriteBundle {
            material: materials.add(background_handle.into()),
            transform: Transform::from_translation(Vec3::new(320., -240., 0.)),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
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
                    transform: Transform::from_translation(Vec3::new(0., 0., 1.)),
                    ..Default::default()
                })
                .with(LoadingText);
        });

    commands.insert_resource(LoadingScreen {
        entity: commands.current_entity().unwrap(),
    });

    for filename in ANIMATION_LIST {
        let path = "data/ANI/".to_owned() + filename;
        assets_loading
            .animations
            .push(asset_server.load(path.as_str()));
        assets_loading.initial_count += 1;
    }

    for filename in SOUND_LIST {
        let path = "data/SOUND/".to_owned() + filename;
        let filename = (*filename).to_owned();
        assets_loading
            .sounds
            .insert(filename, asset_server.load(path.as_str()));
        assets_loading.initial_count += 1;
    }

    assets_loading.schemes = asset_server
        .load_folder("data/SCHEMES")
        .unwrap()
        .drain(..)
        .map(|handle| handle.typed())
        .collect();

    assets_loading.initial_count += assets_loading.schemes.len();
}

fn load_animations(
    mut assets_loading: ResMut<AssetsLoading>,
    mut named_assets: ResMut<NamedAssets>,
    animation_bundle_assets: Res<Assets<AnimationBundle>>,
) {
    let done = assets_loading
        .animations
        .drain_filter(|handle| animation_bundle_assets.contains(handle.clone()));

    for handle in done {
        let bundle = animation_bundle_assets.get(handle).unwrap();
        for (name, animation_handle) in bundle.animations.iter() {
            named_assets
                .animations
                .insert(name.clone(), animation_handle.clone());
        }
    }
}

fn load_sounds(
    mut assets_loading: ResMut<AssetsLoading>,
    mut named_assets: ResMut<NamedAssets>,
    sound_assets: Res<Assets<AudioSource>>,
) {
    let done = assets_loading
        .sounds
        .drain_filter(|_filename, handle| sound_assets.contains(handle.clone()));

    for (filename, handle) in done {
        let name = Path::new(&filename)
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_lowercase();
        named_assets.sounds.insert(name, handle);
    }
}

fn load_schemes(
    mut assets_loading: ResMut<AssetsLoading>,
    mut named_assets: ResMut<NamedAssets>,
    scheme_assets: Res<Assets<Scheme>>,
) {
    let done = assets_loading
        .schemes
        .drain_filter(|handle| scheme_assets.contains(handle.clone()));

    for handle in done {
        let scheme = scheme_assets.get(handle.clone()).unwrap();
        named_assets.schemes.insert(scheme.name.clone(), handle);
    }
}

fn loading_progress(
    mut state: ResMut<State<AppState>>,
    assets_loading: ResMut<AssetsLoading>,
    mut query: Query<&mut Text, With<LoadingText>>,
) {
    let remaining_count = assets_loading.animations.len()
        + assets_loading.sounds.len()
        + assets_loading.schemes.len();
    let assets_loaded = assets_loading.initial_count - remaining_count;

    for mut text in query.iter_mut() {
        text.value = format!(
            "Loading {}/{} assets",
            assets_loaded, assets_loading.initial_count
        );
    }

    if assets_loaded == assets_loading.initial_count {
        state.set_next(AppState::Game).unwrap();
    }
}

fn cleanup(commands: &mut Commands, loading_screen: Res<LoadingScreen>) {
    // trace!("very noisy");
    // debug!("helpful for debugging");
    // info!("helpful information that is worth printing by default");
    // warn!("something bad happened that isn't a failure, but thats worth calling out");
    // error!("something failed");

    commands.despawn_recursive(loading_screen.entity);
}
