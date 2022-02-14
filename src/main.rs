use std::time::Duration;

use bevy::{prelude::*, reflect::TypeUuid, utils::HashMap};

use serde::{Deserialize, Serialize};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(spawn)
        .add_system(animate_sprite_system)
        .run();
}

const BOMB_METADATA: &str = include_str!("../assets/basic_bomb.json");

fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let sprite: aseprite::SpriteInfo<bomb::BombState> =
        serde_json::from_str(BOMB_METADATA).unwrap();
    let tile_size = &sprite.frames.iter().next().unwrap().source_size;
    let texture_handle = asset_server.load(&sprite.meta.image);
    let dimensions = Vec2::new(tile_size.w as f32, tile_size.h as f32);

    let layer_info = sprite.extract_layer_info();
    commands
        .spawn_bundle(bomb::BombBundle {
            bomb_state: bomb::BombState::Fuse,
            ..Default::default()
        })
        .with_children(|builder| {
            for (key, val) in layer_info {
                info!("Spawning layer {key:?}");
                // Each layer gets its own texture atlas to iterate through!
                let mut texture_atlas = TextureAtlas::new_empty(texture_handle.clone(), dimensions);
                for (_bomb_state, (_tags, frames)) in (val.0).iter() {
                    for frame in frames {
                        let rect = bevy::sprite::Rect {
                            min: Vec2::new(frame.frame.x as f32, frame.frame.y as f32),
                            max: Vec2::new(
                                (frame.frame.x + frame.frame.w) as f32,
                                (frame.frame.y + frame.frame.h) as f32,
                            ),
                        };
                        texture_atlas.add_texture(rect);
                    }
                }
                let texture_atlas = textures.add(texture_atlas);
                builder
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas,
                        transform: Transform::from_scale(Vec3::splat(30.0)),
                        ..Default::default()
                    })
                    .insert_bundle(bomb::LayerBundle {
                        layer_name: key,
                        sprites: val,
                    })
                    .insert(Play);
            }
        });
}

pub fn animate_sprite_system(
    time: Res<Time>,
    mut query_bomb: Query<(&bomb::BombState, &Children, &mut AnimationTimer)>,
    mut query_layers: Query<
        (
            &mut TextureAtlasSprite,
            &AnimationLayerInfo<bomb::BombState>,
        ),
        With<Play>,
    >,
) {
    for (bomb_state, children, mut timer) in query_bomb.iter_mut() {
        let children: &Children = children;
        let timer: &mut AnimationTimer = &mut timer;
        timer.0.tick(time.delta());
        for &child in children.iter() {
            if let Ok((mut sprite, animation_info)) = query_layers.get_mut(child) {
                let sprite: &mut TextureAtlasSprite = &mut sprite;
                let animation_info: &AnimationLayerInfo<bomb::BombState> = animation_info;

                if timer.0.finished() {
                    let (tag, frames) = animation_info.0.get(bomb_state).unwrap();
                    let length = tag.from + tag.to + 1;
                    let next_frame = ((sprite.index as usize + 1) % length) as usize;
                    sprite.index = next_frame;

                }
            }

            // TODO hold the time delta frame info for the query_bomb object
            //      Dynamically adjust the time delta there as needed.
        }
    }
}

#[derive(Component)]
pub struct Play;

#[derive(Component)]
pub struct Layer;

use bevy::utils::HashMap as BevyHashMap;

#[derive(Component, Debug, Default)]
pub struct AnimationLayerInfo<T>(BevyHashMap<T, (aseprite::TagInfo<T>, Vec<aseprite::FrameInfo>)>);

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, true))
    }
}

pub mod bomb {
    use super::*;
    use strum_macros::EnumString;

    #[derive(Bundle, Default)]
    pub struct BombBundle {
        pub bomb_state: BombState,
        pub animation_timer: AnimationTimer,
    }

    #[derive(Bundle, Default)]
    pub struct LayerBundle {
        pub layer_name: aseprite::LayerName,
        pub sprites: AnimationLayerInfo<BombState>,
    }

    #[derive(Component, Debug, Serialize, Deserialize, Hash, Clone, PartialEq, Eq, EnumString)]
    pub enum BombState {
        #[strum(ascii_case_insensitive)]
        #[serde(rename = "IDLE")]
        Idle,
        #[strum(ascii_case_insensitive)]
        #[serde(rename = "FUSE")]
        Fuse,
    }

    impl Default for BombState {
        fn default() -> Self {
            BombState::Idle
        }
    }
}

// use bevy::utils::HashMap as BevyHashMap;
// #[derive(Component, Debug, Default)]
// pub struct AnimationInfo<T>(BevyHashMap<T, Vec<(aseprite::LayerName, Vec<aseprite::FrameInfo>)>>);

pub mod aseprite {
    use bevy::{prelude::Component, utils::HashMap as BevyHashMap};
    use std::{collections::HashMap, hash::Hash, str::FromStr};

    use serde::{de::DeserializeOwned, Deserialize};

    use crate::AnimationLayerInfo;

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SpriteInfo<AnimationName> {
        pub frames: Vec<FrameInfo>,
        pub meta: Meta<AnimationName>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameInfo {
        pub filename: String,
        pub frame: SizeInfo,
        pub rotated: bool,
        pub trimmed: bool,
        pub sprite_source_size: SizeInfo,
        pub source_size: SizeInfoMin,
        pub duration: usize,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SizeInfo {
        pub x: usize,
        pub y: usize,
        pub w: usize,
        pub h: usize,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SizeInfoMin {
        pub w: usize,
        pub h: usize,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Meta<AnimationName> {
        pub app: String,
        pub version: String,
        pub image: String,
        pub format: String,
        pub size: SizeInfoMin,
        pub scale: String,
        pub frame_tags: Vec<TagInfo<AnimationName>>,
        pub layers: Vec<LayerInfo>,
        pub slices: Vec<String>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct LayerInfo {
        pub name: String,
        pub opacity: usize,
        pub blend_mode: String,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct TagInfo<AnimationName> {
        pub name: AnimationName,
        pub from: usize,
        pub to: usize,
        pub direction: String,
    }

    #[derive(Component, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
    pub struct LayerName(pub String);

    impl<AnimationName> SpriteInfo<AnimationName>
    where
        AnimationName: std::cmp::Eq + Hash + Clone + FromStr,
    {
        pub fn extract_layer_info(
            self,
        ) -> BevyHashMap<LayerName, AnimationLayerInfo<AnimationName>> {
            let frames = self.frames.into_iter().fold::<BevyHashMap<
                LayerName,
                AnimationLayerInfo<AnimationName>,
            >, _>(
                BevyHashMap::default(),
                |mut acc, frame| {
                    let mut split = frame.filename.split("-");
                    let animation_name = split.next().expect("Could not parse animation name");
                    println!("{animation_name:?}");
                    let animation_name = AnimationName::from_str(&animation_name);
                    let animation_name = animation_name
                        .map_err(|_| "Invalid animation name")
                        .unwrap();

                    let layer = split
                        .next()
                        .expect("Could not parse layer info")
                        .to_string();
                    let layer = LayerName(layer);
                    let frame_num = split.next().expect("Could not parse frame number");

                    let animation_map = if let Some(animation_map) = acc.get_mut(&layer) {
                        animation_map
                    } else {
                        let animation_map = AnimationLayerInfo(BevyHashMap::default());
                        acc.insert(layer.clone(), animation_map);
                        acc.get_mut(&layer).unwrap()
                    };

                    if let Some((_tag, frames)) = animation_map.0.get_mut(&animation_name) {
                        frames.push(frame);
                    } else {
                        let tag = self
                            .meta
                            .frame_tags
                            .iter()
                            .find(|e| e.name == animation_name)
                            .expect("The meta tags do not include a layer specified in frames!");
                        animation_map
                            .0
                            .insert(animation_name, (tag.clone(), vec![frame]));
                    }
                    acc
                },
            );
            return frames;
        }
    }
}
