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
    let tile_size = &sprite.frames.get(0).unwrap().source_size;
    let texture_handle = asset_server.load(&sprite.meta.image);
    let dimensions = Vec2::new(tile_size.w as f32, tile_size.h as f32);

    let animation_tag_info = sprite.clone().extract_animation_tag_info();
    let layer_info = sprite.extract_layer_info();
    commands
        .spawn_bundle(bomb::BombBundle {
            bomb_state: bomb::BombState::Fuse,
            animation_tag_info,
            ..Default::default()
        })
        .with_children(|builder| {
            for (relative_index, frames, layer_name, supported_animations) in layer_info.into_iter()
            {
                info!("Spawning layer {layer_name:?} on idx {relative_index:?}");
                // Each layer gets its own texture atlas to iterate through!
                let mut texture_atlas = TextureAtlas::new_empty(texture_handle.clone(), dimensions);
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
                let texture_atlas = textures.add(texture_atlas);
                builder
                    .spawn_bundle(SpriteSheetBundle {
                        texture_atlas,
                        global_transform: GlobalTransform {
                            translation: Vec3::new(0., 0., relative_index as f32),
                            ..Default::default()
                        },
                        visibility: Visibility {
                            is_visible: false,
                        },
                        ..Default::default()
                    })
                    .insert_bundle(bomb::LayerBundle {
                        name: layer_name,
                        supported_animations,
                    });
            }
        });
}

pub fn animate_sprite_system(
    time: Res<Time>,
    mut query_bomb: Query<(
        &bomb::BombState,
        &Children,
        &mut AnimationTimer,
        &AnimationLayerInfo<bomb::BombState>,
    )>,
    mut query_layers: Query<(
        &mut TextureAtlasSprite,
        &mut Visibility,
        &SupportedAnimations<bomb::BombState>,
    )>,
) {
    for (current_bomb_state, children, mut timer, animation_info) in query_bomb.iter_mut() {
        let children: &Children = children;
        let timer: &mut AnimationTimer = &mut timer;
        let animation_info: &AnimationLayerInfo<bomb::BombState> = animation_info;
        timer.0.tick(time.delta());

        let tag = animation_info.0.get(current_bomb_state).unwrap(); // All animation tags are registered!
        for &child in children.iter() {
            if let Ok((mut sprite, mut visibility, supported_animations)) =
                query_layers.get_mut(child)
            {
                let sprite: &mut TextureAtlasSprite = &mut sprite;
                let visibility: &mut Visibility = &mut visibility;
                let supported_animations: &SupportedAnimations<bomb::BombState> =
                    supported_animations;
                if timer.0.finished() {
                    if supported_animations.0.contains(current_bomb_state) {
                        let length = tag.to - tag.from + 1;
                        let next_frame = ((sprite.index as usize + 1) % length) as usize;
                        sprite.index = next_frame;
                        visibility.is_visible = true;
                    } else {
                        visibility.is_visible = false;
                    }
                }
            }
        }
    }
}

#[derive(Component)]
pub struct Layer;

use bevy::utils::HashMap as BevyHashMap;
use bevy::utils::HashSet as BevyHashSet;

#[derive(Component, Debug, Default)]
pub struct AnimationLayerInfo<T>(BevyHashMap<T, aseprite::TagInfo<T>>);

#[derive(Component, Debug, Default)]
pub struct SupportedAnimations<T>(BevyHashSet<T>);

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
        pub animation_tag_info: AnimationLayerInfo<bomb::BombState>,
    }

    #[derive(Bundle, Default)]
    pub struct LayerBundle {
        pub name: aseprite::LayerName,
        pub supported_animations: SupportedAnimations<bomb::BombState>,
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

pub mod aseprite {
    use bevy::{
        prelude::Component,
        utils::{HashMap as BevyHashMap, HashSet},
    };
    use std::{hash::Hash, str::FromStr};

    use serde::Deserialize;

    use crate::{AnimationLayerInfo, SupportedAnimations};
    use strum_macros::EnumString;

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SpriteInfo<AnimationName> {
        pub frames: Vec<FrameInfo>,
        pub meta: Meta<AnimationName>,
    }

    #[derive(Deserialize, Debug, Clone)]
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

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SizeInfo {
        pub x: usize,
        pub y: usize,
        pub w: usize,
        pub h: usize,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SizeInfoMin {
        pub w: usize,
        pub h: usize,
    }

    #[derive(Deserialize, Debug, Clone)]
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

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct LayerInfo {
        pub name: LayerName,
        pub opacity: usize,
        pub blend_mode: String,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct TagInfo<AnimationName> {
        pub name: AnimationName,
        pub from: usize,
        pub to: usize,
        // TODO: currently there's no support for each enum. Everything is only of 'forward' type
        pub direction: AnimationDirections,
    }


    #[derive(Component, Debug, Deserialize, Hash, Clone, PartialEq, Eq, EnumString)]
    pub enum AnimationDirections {
        #[strum(ascii_case_insensitive)]
        #[serde(rename = "forward")]
        Forward,
        #[strum(ascii_case_insensitive)]
        #[serde(rename = "backward")]
        Backward,
        #[strum(ascii_case_insensitive)]
        #[serde(rename = "pingpong")]
        PingPong,
    }

    #[derive(
        Component, Default, Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Deserialize,
    )]
    pub struct LayerName(pub String);

    impl<AnimationName> SpriteInfo<AnimationName>
    where
        AnimationName: std::cmp::Eq + Hash + Clone + FromStr,
    {
        pub fn extract_animation_tag_info(self) -> AnimationLayerInfo<AnimationName> {
            let tags =
                self.meta
                    .frame_tags
                    .into_iter()
                    .fold(BevyHashMap::default(), |mut acc, tag| {
                        acc.insert(tag.name.clone(), tag);
                        acc
                    });
            return AnimationLayerInfo(tags);
        }

        pub fn extract_layer_info(
            self,
        ) -> Vec<(
            usize,
            Vec<FrameInfo>,
            LayerName,
            SupportedAnimations<AnimationName>,
        )> {
            let mut hm: BevyHashMap<LayerName, (usize, Vec<FrameInfo>, HashSet<AnimationName>)> =
                BevyHashMap::default();
            for frame in self.frames {
                let (animation_name, layer_name) = Self::extract_frame_info(&frame.filename);
                if let Some((_idx, frames, animations)) = hm.get_mut(&layer_name) {
                    frames.push(frame);
                    animations.insert(animation_name);
                } else {
                    let mut animation_set = HashSet::default();
                    animation_set.insert(animation_name);
                    let idx = self
                        .meta
                        .layers
                        .iter()
                        .position(|e| e.name == layer_name)
                        .expect(
                            "There's an existing animation frame for a layer that does not exist!",
                        );
                    hm.insert(layer_name, (idx + 1, vec![frame], animation_set));
                }
            }
            let mut result = hm.into_iter().fold(
                vec![],
                |mut acc, (key, (idx, frames, supported_animations))| {
                    acc.push((idx, frames, key, SupportedAnimations(supported_animations)));
                    acc
                },
            );
            result
        }

        fn extract_frame_info(frame_filename: &str) -> (AnimationName, LayerName) {
            let mut split = frame_filename.split('-');
            let animation_name = split.next().expect("Could not parse animation name");
            println!("{animation_name:?}");
            let animation_name = AnimationName::from_str(animation_name);
            let animation_name = animation_name
                .map_err(|_| "Invalid animation name")
                .unwrap();

            let layer = split
                .next()
                .expect("Could not parse layer info")
                .to_string();
            let layer = LayerName(layer);
            let _frame_num = split.next().expect("Could not parse frame number");

            (animation_name, layer)
        }
    }
}
