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

const BOMB_METADATA: &'static str = include_str!("../assets/basic_bomb.json");

fn spawn(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let sprite: aseprite::SpriteInfo<bomb::BombState> =
        serde_json::from_str(BOMB_METADATA).unwrap();
    let tile_size = &sprite.frames.iter().next().unwrap().source_size;

    let columns = sprite.meta.frame_tags.get(0).unwrap().to + 1;
    println!("{:?}", sprite);
    let texture_atlas = TextureAtlas::from_grid(
        asset_server.load(&sprite.meta.image),
        Vec2::new(tile_size.w as f32, tile_size.h as f32),
        columns,
        sprite.meta.layers.len(),
    );
    let texture = textures.add(texture_atlas);

    let bomb_sprite_data = sprite.into();
    let mut entity = commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture,
            transform: Transform::from_scale(Vec3::splat(1.0)),
            ..Default::default()
        })
        .insert_bundle(bomb::BombBundle {
            sprites: bomb_sprite_data,
            bomb_state: bomb::BombState::Fuse,
            ..Default::default()
        }).insert(Play);
}

pub fn animate_sprite_system(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
        &bomb::BombState,
        &mut AnimationTick,
        &AnimationInfo<bomb::BombState>,
    ), With<Play>>,
) {
    for (mut timer, mut sprite, texture_atlas_handle, bomb_state, mut animation_tick, animation_info) in query.iter_mut() {
        let timer: &mut AnimationTimer = &mut timer;
        let sprite: &mut TextureAtlasSprite = &mut sprite;
        let texture_atlas_handle: &Handle<TextureAtlas> = texture_atlas_handle;
        let animation_tick: &mut AnimationTick = &mut animation_tick;
        let animation_info: &AnimationInfo<bomb::BombState> = animation_info;

        timer.0.tick(time.delta());
        if timer.0.finished() {
            // texture_atlas_handle.
            let (_layer, animations) = animation_info.0.get(bomb_state).unwrap().first().unwrap();
            let length = animations.len();
            let next_frame = ((sprite.index as usize + 1) % length) as usize;
            sprite.index = next_frame;
        }
    }
}


// fn animate(
//     time: Res<Time>,
//     mut query: Query<(
//         &mut AnimationTimer,
//         &mut TextureAtlasSprite,
//         &bomb::BombState,
//         &mut AnimationTick,
//         &SpriteAtlas<bomb::BombState>,
//     )>,
//     anim_handles: Res<AnimationHandles>,
//     anim_data_assets: Res<Assets<AnimationData>>,
// ) {

//     for (mut timer, mut sprite, texture_atlas_handle, must_animate) in query.iter_mut() {
//         timer.0.tick(time.delta());
//         if timer.0.finished() {
//             let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
//             sprite.index = if must_animate.0 {
//                 ((sprite.index as usize + 1) % texture_atlas.textures.len()) as usize
//             } else {
//                 0
//             };
//         }
//     }
// }


#[derive(Component)]
pub struct Play;

pub mod bomb {
    use super::*;
    use strum_macros::EnumString;

    #[derive(Bundle, Default)]
    pub struct BombBundle {
        pub animation_tick: AnimationTick,
        pub animation_timer: AnimationTimer,
        pub bomb_state: BombState,
        pub sprites: AnimationInfo<BombState>,
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

use bevy::utils::HashMap as BevyHashMap;
#[derive(Component, Debug, Default)]
pub struct AnimationInfo<T>(BevyHashMap<T, Vec<(aseprite::LayerName, Vec<aseprite::FrameInfo>)>>);

#[derive(Component, Default)]
pub struct AnimationTick(pub u32);

#[derive(Component)]
pub struct AnimationTimer(pub Timer);

impl Default for AnimationTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, true))
    }
}

pub mod aseprite {
    use bevy::utils::HashMap as BevyHashMap;
    use strum_macros::EnumString;
    use std::{collections::HashMap, hash::Hash, str::FromStr};

    use serde::{de::DeserializeOwned, Deserialize};

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

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct TagInfo<AnimationName> {
        pub name: AnimationName,
        pub from: usize,
        pub to: usize,
        pub direction: String,
    }

    #[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone)]
    pub struct LayerName(String);

    impl<AnimationName> From<SpriteInfo<AnimationName>> for crate::AnimationInfo<AnimationName>
    where
        AnimationName: std::cmp::Eq + Hash + Clone + FromStr,
    {
        fn from(item: SpriteInfo<AnimationName>) -> Self {
            let frames = item.frames.into_iter().fold::<BevyHashMap<
                AnimationName,
                Vec<(LayerName, Vec<FrameInfo>)>,
            >, _>(
                BevyHashMap::default(),
                |mut acc, frame| {
                    let mut split = frame.filename.split("-");
                    let animation_name = split.next().unwrap();
                    println!("{animation_name:?}");
                    let animation_name = AnimationName::from_str(&animation_name);
                    let animation_name  = animation_name.map_err(|_| "Invalid animation name").unwrap();

                    let layer = split.next().unwrap().to_string();
                    let layer = LayerName(layer);
                    let frame_num = split.next().unwrap();

                    let animation_map = if let Some(animation_map) = acc.get_mut(&animation_name) {
                        animation_map
                    } else {
                        let animation_map = vec![];
                        acc.insert(animation_name.clone(), animation_map);
                        acc.get_mut(&animation_name).unwrap()
                    };

                    if let Some((_, frames)) = animation_map.iter_mut().find(|e| e.0 == layer) {
                        frames.push(frame);
                    } else {
                        animation_map.push((layer, vec![frame]))
                    }
                    acc
                },
            );
            Self(frames)
        }
    }
}
