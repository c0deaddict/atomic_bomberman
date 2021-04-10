use crate::animation::Animation;
use crate::asset_loaders::Scheme;
use bevy::prelude::*;
use std::collections::HashMap;
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;

pub const STAGE: &str = "app_state";

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
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
