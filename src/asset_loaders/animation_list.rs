use anyhow::Result;
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use std::fmt::Debug;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Default)]
pub struct AnimationListAssetLoader;

#[derive(Debug, TypeUuid)]
#[uuid = "5e81abab-2517-4b93-acf0-2e4ecbb56881"]
pub struct AnimationList(Vec<String>);

impl AssetLoader for AnimationListAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            let animation_list = parse(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(animation_list));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ali", "ALI"]
    }
}

fn parse(bytes: &[u8]) -> Result<AnimationList> {
    let mut reader = BufReader::new(bytes);
    let mut list = Vec::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(AnimationList(list));
        }

        let line = line.trim();
        if let Some(filename) = line.strip_prefix('-') {
            list.push(filename.to_owned());
        }
    }
}
