use anyhow::{bail, Result};
use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use std::fmt::Debug;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Default)]
pub struct SchemeAssetLoader;

#[derive(Copy, Clone)]
pub enum Powerup {
    ExtraBomb = 0,
    LongerFlame = 1,
    Disease = 2,
    Kick = 3,
    Speed = 4,
    Punch = 5,
    Grab = 6,
    Spooger = 7,
    Goldflame = 8,
    Trigger = 9,
    Jelly = 10,
    SuperBadDisease = 11,
    Random = 12,
}

#[derive(Debug, Copy, Clone)]
pub enum Cell {
    Solid,
    Brick,
    Blank,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Blank
    }
}

pub type Grid = [[Cell; 15]; 11];

#[derive(Debug, Default)]
pub struct PowerupInfo {
    pub born_with: bool,
    pub override_value: Option<i8>,
    pub forbidden: bool,
}

#[derive(Debug, Default, TypeUuid)]
#[uuid = "cc0c8d4b-e8cb-49f9-8baf-7974a84c4172"]
pub struct Scheme {
    pub version: u8,
    pub name: String,
    pub brick_density: u8,
    pub grid: Grid,
    pub player_start_locs: [(u8, u8); 10], // x, y
    pub powerup_infos: [PowerupInfo; 13],
}

impl Scheme {
    fn new() -> Scheme {
        Scheme {
            ..Default::default()
        }
    }
}

impl AssetLoader for SchemeAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(parse(bytes)?));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["sch", "SCH"]
    }
}

fn parse(bytes: &[u8]) -> Result<Scheme> {
    let mut reader = BufReader::new(bytes);
    let mut scheme = Scheme::new();

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            return Ok(scheme);
        }

        let parts = match line.trim().strip_prefix('-') {
            Some(line) => line.split(',').map(|s| s.trim()).collect::<Vec<_>>(),
            None => continue,
        };

        match parts[0] {
            "V" => scheme.version = parts[1].parse().unwrap(),
            "N" => scheme.name = parts[1].to_owned(),
            "B" => scheme.brick_density = parts[1].parse().unwrap(),
            "R" => {
                let row: usize = parts[1].parse().unwrap();
                parts[2]
                    .chars()
                    .map(|c| match c {
                        '#' => Cell::Solid,
                        ':' => Cell::Brick,
                        _ => Cell::Blank,
                    })
                    .enumerate()
                    .for_each(|(col, cell)| {
                        scheme.grid[row][col] = cell;
                    });
            }
            "S" => {
                let num: usize = parts[1].parse().unwrap();
                let x = parts[2].parse().unwrap();
                let y = parts[3].parse().unwrap();
                scheme.player_start_locs[num] = (x, y);
            }
            "P" => {
                let index: usize = parts[1].parse().unwrap();
                scheme.powerup_infos[index] = PowerupInfo {
                    born_with: parts[2].parse::<u8>().unwrap() != 0,
                    override_value: if parts[3].parse::<u8>().unwrap() != 0 {
                        Some(parts[4].parse().unwrap())
                    } else {
                        None
                    },
                    forbidden: parts[5].parse::<u8>().unwrap() != 0,
                };
            }
            key => bail!("Unknown scheme attribute {}", key),
        }
    }
}
