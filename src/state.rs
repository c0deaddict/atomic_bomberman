use crate::asset_loaders::{Animation, Scheme};
use bevy::prelude::*;
use std::collections::HashMap;

pub const STAGE: &str = "app_state";

#[derive(Clone)]
pub enum AppState {
    Loading,
    Game,
}

#[derive(Default)]
pub struct NamedAssets {
    pub animations: HashMap<String, Handle<Animation>>,
    pub sounds: HashMap<String, Handle<AudioSource>>,
    pub schemes: HashMap<String, Handle<Scheme>>,
}
