use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::utils::animations::{AnimationIndices, FrameTimer};
use crate::utils::click_plugin::Clickable;
use crate::{GameState, utils::game_assets::GameAssets};
use crate::world::tiled::*;

pub struct MapPlugin;

impl Plugin for MapPlugin 
{
    fn build(&self, app: &mut App) 
    {
        app.add_systems(OnEnter(GameState::Playing), setup_map);
    }
}

fn setup_map(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
)
{
    for (y, row) in GRASS.iter().enumerate() 
    {
        for (x, &grass_value) in row.iter().enumerate() 
        {
            let pos = tile_to_world(x, y);

            commands.spawn(
                (Sprite::from_atlas_image(
                    game_assets.water_texture.clone(), 
                    TextureAtlas 
                    { 
                        layout: game_assets.water_layout.clone(),
                        index: WATER_INDEX,
                    }
                ), 
                AnimationIndices { first: 0, last: 3 },
                FrameTimer(Timer::from_seconds(0.5, TimerMode::Repeating)),
                Transform::from_xyz(pos.x, pos.y, 0.0)
            ));

            if grass_value >= 0
            {
                commands.spawn(
                    (Sprite::from_atlas_image(
                        game_assets.grass_texture.clone(), 
                        TextureAtlas 
                        { 
                            layout: game_assets.grass_layout.clone(),
                            index: grass_value as usize,
                        }
                    ), 
                    Transform::from_xyz(pos.x, pos.y, 1.0)
                ));
            }
        }
    }

    for (y, row) in BIG_BIOME.iter().enumerate() 
    {
        for (x, &biome_value) in row.iter().enumerate() 
        {
            if biome_value >= 0
            {
                let pos = tile_to_world(x, y);

                commands.spawn(
                    (Sprite::from_atlas_image(
                        game_assets.big_biome_texture.clone(), 
                        TextureAtlas 
                        { 
                            layout: game_assets.big_biome_layout.clone(),
                            index: biome_value as usize,
                        }
                    ), 
                    Transform::from_xyz(pos.x, pos.y, 4.0)
                ));
            }
        }
    }

    for (y, row) in SMALL_BIOME.iter().enumerate() 
    {
        for (x, &biome_value) in row.iter().enumerate() 
        {
            if biome_value >= 0
            {
                let pos = tile_to_world(x, y);

                commands.spawn(
                    (Sprite::from_atlas_image(
                        game_assets.big_biome_texture.clone(), 
                        TextureAtlas 
                        { 
                            layout: game_assets.small_biome_layout.clone(),
                            index: biome_value as usize,
                        }
                    ), 
                    Transform::from_xyz(pos.x, pos.y, 2.0),
                    RigidBody::Fixed,
                    Collider::cuboid(TILE_SIZE / 2.0, TILE_SIZE / 4.0, 0.1),

                    Clickable,
                ));
            }
        }
    }
}

pub fn tile_to_world(x: usize, y: usize) -> Vec2
{
    Vec2::new(
        (x as f32 - (MAP_WIDTH - 1) as f32 / 2.0) * TILE_SIZE,
        ((MAP_HEIGHT - 1) as f32 / 2.0 - y as f32) * TILE_SIZE,
    )
}