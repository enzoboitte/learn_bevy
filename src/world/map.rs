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
                AnimationIndices {  mode: TimerMode::Repeating, start: WATER_INDEX, first: 0, last: 3 },
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

                add_entity_to_map(pos, commands.spawn(
                    (Sprite::from_atlas_image(
                        game_assets.big_biome_texture.clone(), 
                        TextureAtlas 
                        { 
                            layout: game_assets.big_biome_layout.clone(),
                            index: biome_value as usize,
                        }
                    ), 
                    Transform::from_xyz(pos.x, pos.y, 4.0)
                )).id());
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

                add_entity_to_map(pos, commands.spawn(
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
                )).id());
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

pub fn world_to_tile(pos: Vec2) -> Option<(usize, usize)>
{
    let x = ((pos.x / TILE_SIZE) + (MAP_WIDTH - 1) as f32 / 2.0).round() as isize;
    let y = (((MAP_HEIGHT - 1) as f32 / 2.0) - (pos.y / TILE_SIZE)).round() as isize;

    if x >= 0 && x < MAP_WIDTH as isize && y >= 0 && y < MAP_HEIGHT as isize 
    {
        Some((x as usize, y as usize))
    } 
    else 
    {
        None
    }
}

pub fn is_tile_occupied(position: Vec2) -> bool
{
    print!("Checking if tile is occupied at position: {:?}", position);
    if let Some((x, y)) = world_to_tile(position)
    {
        println!("Converted to tile coordinates: ({}, {})", x, y);
        let ret = ENTITY_MAP.read().unwrap().contains_key(&(x, y));
        println!("Tile occupied: {}", ret);

        ret
    } else {
        false
    }
}

pub fn add_entity_to_map(position: Vec2, entity: Entity)
{
    if let Some((x, y)) = world_to_tile(position)
    {
        ENTITY_MAP.write().unwrap().insert((x, y), entity);
    }
}

pub fn remove_entity_from_map(position: Vec2)
{
    if let Some((x, y)) = world_to_tile(position)
    {
        ENTITY_MAP.write().unwrap().remove(&(x, y));
    }
}

pub fn remove_entity_from_map_by_id(entity: Entity)
{
    let mut entity_map = ENTITY_MAP.write().unwrap();
    let key_to_remove = entity_map.iter()
        .find(|(_, e)| e.index() == entity.index())
        .map(|(&key, _)| key);

    if let Some(key) = key_to_remove 
    {
        entity_map.remove(&key);
    }
}