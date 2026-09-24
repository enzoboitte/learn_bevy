use bevy::prelude::*;

use crate::entities::player::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin 
{
    fn build(&self, app: &mut App) 
    {
        app.add_systems(Update, animate_sprites);
    }
}

#[derive(Message)]
pub struct AnimationStep
{
    pub entity: Entity,
    pub index: usize,
}

#[derive(Component)]
pub struct AnimationIndices 
{
    pub mode: TimerMode,
    pub start: usize,
    pub first: usize,
    pub last: usize,
}

#[derive(Component)]
pub struct FrameTimer(pub Timer);

pub fn animate_sprites(
    time: Res<Time>,
    mut query: Query<(Entity, &AnimationIndices, &mut FrameTimer, &mut Sprite)>,

    mut writer: MessageWriter<AnimationStep>,
)
{
    for (entity, indices, mut timer, mut sprite) in query.iter_mut() 
    {
        timer.0.tick(time.delta());

        if timer.0.just_finished() 
        {
            if indices.mode == TimerMode::Once && sprite.texture_atlas.as_ref().unwrap().index == indices.last
            { continue; }
            if let Some(atlas) = &mut sprite.texture_atlas 
            {
                writer.write(AnimationStep 
                { 
                    entity: entity, 
                    index: atlas.index - indices.start, 
                });
                atlas.index = if atlas.index == indices.last 
                {
                    indices.first
                } else 
                {
                    atlas.index + 1
                };
            }
        }   
    }
}

pub fn player_sprite_indices(state: &PlayerState, direction: &PlayerDirection) -> (usize, usize)
{
    match state 
    {
        PlayerState::Idle => match direction
        {
            PlayerDirection::Right => (30, 31),
            PlayerDirection::Left => (20, 21),
            PlayerDirection::Up => (10, 11),
            _ => (0, 1)
        },
        PlayerState::Walking => match direction
        {
            PlayerDirection::Right => (32, 33),
            PlayerDirection::Left => (22, 23),
            PlayerDirection::Up => (12, 13),
            _ => (2, 3)
        }
    }
}