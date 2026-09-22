use bevy::prelude::*;

use crate::GameState;

const TILE_SIZE: f32 = 16.0;
const MAP_WIDTH: usize = 16;
const MAP_HEIGHT: usize = 12;
const WATER_INDEX: usize = 0;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_map);
    }
}

fn setup_map(
    mut commands: Commands,
)
{
    commands.spawn((Transform::default(), GlobalTransform::default()));
}