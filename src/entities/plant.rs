use std::{ops::Range, sync::LazyLock};
use bevy::{platform::collections::HashMap, prelude::*};
use bevy_rapier3d::prelude::*;

use crate::{GameState, utils::{animations::{AnimationIndices, AnimationStep, FrameTimer}, game_assets::GameAssets}, world::{map::{add_entity_to_map, world_to_tile}, tiled::ENTITY_MAP}};

pub struct PlantPlugin;

impl Plugin for PlantPlugin 
{
    fn build(&self, app: &mut App) 
    {
        app
            .add_message::<SpawnPlant>()
            .add_message::<AnimationStep>()
            .add_systems(Update, (spawn_plant).run_if(in_state(GameState::Playing)))
            .add_systems(FixedUpdate, (update_plants).run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Plant
{
    pub plant_type: PlantType,
    pub state: PlantState,
    pub start_time: f32,
    pub growth_timer: f32,
    pub range: Range<usize>,
}

#[derive(Message)]
pub struct SpawnPlant {
    pub plant_type: PlantType,
    pub plant_state: PlantState,
    pub position: Vec3,
    pub growth_duration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlantType 
{
    WHEAT,
    TOMATO,
}

static INDEX_PLANT_TYPE: LazyLock<HashMap<PlantType, Range<usize>>> = 
LazyLock::new(|| HashMap::from([
    (PlantType::WHEAT, 1..4),
    (PlantType::TOMATO, 7..10),
]));

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlantState  
{
    FIRSTGROWTH = 1,
    SECONDGROWTH = 2,
    THIRDGROWTH = 3,
    MATURE = 4,
}

fn spawn_plant(
    mut commands: Commands,
    game_assets: Res<GameAssets>,

    mut message: MessageReader<SpawnPlant>,
    time: Res<Time>,
)
{
    for spawn_plant in message.read() 
    {
        let start_time = time.elapsed_secs();
        let range = INDEX_PLANT_TYPE.get(&spawn_plant.plant_type).unwrap();

        add_entity_to_map(spawn_plant.position.truncate(), commands.spawn((
            Sprite::from_atlas_image(
                game_assets.plant_texture.clone(),
                TextureAtlas 
                {
                    layout: game_assets.plant.clone(),
                    index: range.start + spawn_plant.plant_state as usize - 1,
                },
            ),
            AnimationIndices 
            {
                mode: TimerMode::Once,
                start: range.start,
                first: spawn_plant.plant_state as usize,
                last: range.end,
            },
            FrameTimer(Timer::from_seconds(spawn_plant.growth_duration, TimerMode::Repeating)),
            Transform::from_xyz(spawn_plant.position.x, spawn_plant.position.y, 1.0),
            Plant {
                plant_type: spawn_plant.plant_type,
                state: spawn_plant.plant_state,
                start_time,
                growth_timer: spawn_plant.growth_duration,
                range: range.clone(),
            },

            Velocity::zero(),
            RigidBody::Fixed,
            GravityScale(0.0),
            Collider::cuboid(16.0 / 2.0, 16.0 / 2.0, 0.1),
        ))
        .with_children(|parent| 
            {
                parent.spawn((
                    Text2d::new((spawn_plant.growth_duration * (range.end - range.start) as f32).to_string()),
                    TextFont
                    {
                        font: FontSource::Handle(game_assets.font.clone()),
                        font_size: FontSize::Px(6.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_xyz(0.0, 6.0, 1.0),
                ));
            }
        ).id());
    }
}

fn update_plants(
    mut commands: Commands,
    mut plants: Query<(&mut Plant, &mut AnimationIndices, &mut Children)>,
    mut texts: Query<&mut Text2d>,

    mut message: MessageReader<AnimationStep>,
    time: Res<Time>,
)
{
    for animation_step in message.read() 
    {
        if let Ok((mut indices, _, _)) = plants.get_mut(animation_step.entity) 
        {
            indices.state = match animation_step.index 
            {
                0 => PlantState::SECONDGROWTH,
                1 => PlantState::THIRDGROWTH,
                _ => PlantState::MATURE,
            };
            println!("Plant state updated to: {:?}, {:?}", indices.state, animation_step.index);
        }
    }

    // update text of the plant's growth timer
    for (plant, _, children) in plants
    {
        let base = plant.growth_timer;
        let total = base * (plant.range.end - plant.range.start) as f32;

        let elapsed_time = time.elapsed_secs() - plant.start_time;
        let remaining_time = (total - elapsed_time).max(0.0);

        if let Some(&text_entity) = children.first() 
        {
            if remaining_time <= 0.0 
            {
                commands.entity(text_entity).despawn();
                continue;
            }
            if let Ok(mut text) = texts.get_mut(text_entity) 
            {
                text.0 = format!("{:.1}", remaining_time);
            }
        }
    }
}