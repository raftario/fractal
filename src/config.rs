use crate::colour::Gradient;
use anyhow::Result;
use sdl2::keyboard::Keycode;
use serde::Deserialize;
use std::{
    convert::TryFrom,
    fs,
    path::{Path, PathBuf},
};

pub fn read(path: impl AsRef<Path>) -> Result<Config> {
    let contents = fs::read(path)?;
    let config = toml::from_slice(&contents)?;
    Ok(config)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub max_iterations: usize,
    pub preview: PreviewConfig,
    pub render: RenderConfig,
    pub gradient: Gradient,
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PreviewConfig {
    #[serde(flatten)]
    pub resolution: Resolution,
    pub move_factor: f64,
    pub zoom_factor: f64,
    pub keys: PreviewKeysConfig,
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct PreviewKeysConfig {
    pub up: Key,
    pub right: Key,
    pub down: Key,
    pub left: Key,
    pub zoom_in: Key,
    pub zoom_out: Key,
    pub render: Key,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct RenderConfig {
    #[serde(flatten)]
    pub resolution: Resolution,
    pub directory: PathBuf,
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Resolution {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Copy, Clone, Deserialize)]
#[serde(try_from = "&str")]
pub struct Key(Keycode);

impl Default for Config {
    fn default() -> Self {
        Self {
            max_iterations: 512,
            preview: Default::default(),
            render: Default::default(),
            gradient: Default::default(),
        }
    }
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            resolution: Resolution {
                width: 350,
                height: 200,
            },
            move_factor: 0.125,
            zoom_factor: 1.25,
            keys: Default::default(),
        }
    }
}

impl Default for PreviewKeysConfig {
    fn default() -> Self {
        Self {
            up: Key(Keycode::W),
            right: Key(Keycode::D),
            down: Key(Keycode::S),
            left: Key(Keycode::A),
            zoom_in: Key(Keycode::Up),
            zoom_out: Key(Keycode::Down),
            render: Key(Keycode::R),
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            resolution: Resolution {
                width: 3840,
                height: 2160,
            },
            directory: PathBuf::from("renders"),
        }
    }
}

impl TryFrom<&str> for Key {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let keycode = Keycode::from_name(value).ok_or("unknown SDL2 keycode")?;
        Ok(Self(keycode))
    }
}

impl PartialEq<Keycode> for Key {
    fn eq(&self, other: &Keycode) -> bool {
        self.0.eq(other)
    }
}
