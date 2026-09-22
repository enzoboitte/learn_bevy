use bevy::prelude::*;

use crate::GameState;
use crate::utils::game_assets::GameAssets;
use crate::utils::animations::*;

const PLAYER_SPEED: f32 = 100.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, (move_player, update_indices).run_if(in_state(GameState::Playing)));
    }
}

pub enum PlayerDirection {
    Up,
    Down,
    Left,
    Right
}
pub enum PlayerState {
    Idle,
    Walking
}

#[derive(Component)]
pub struct Player {
    pub current_direction: PlayerDirection,
    pub state: PlayerState,
}

fn spawn_player(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
)
{
    commands.spawn((
        Sprite::from_atlas_image(
            game_assets.player_texture.clone(),
            TextureAtlas 
            {
                layout: game_assets.player.clone(),
                index: 0,
            },
        ),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Player 
        {
            current_direction: PlayerDirection::Down,
            state: PlayerState::Idle,
        },
        AnimationIndices 
        {
            first: 0,
            last: 1,
        },
        FrameTimer(Timer::from_seconds(0.3, TimerMode::Repeating)),
    ));
}

fn update_indices(
    mut query: Query<(&mut AnimationIndices, &mut Sprite, &Player)>
)
{
    for (mut indices, mut sprite, player) in query.iter_mut() 
    {
        let new_indices = player_sprite_indices(&player.state, &player.current_direction);

        if new_indices.0 != indices.first
        {
            indices.first = new_indices.0;
            indices.last = new_indices.1;
            
            if let Some(atlas) = &mut sprite.texture_atlas 
            {
                atlas.index = indices.first;
            }
        }
    }
}

fn move_player(
    time: Res<Time>,
    mut player: Single<(&mut Transform, &mut Player), With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>)
{
    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) 
    {
        direction.y += 1.0;
        player.1.current_direction = PlayerDirection::Up;
    }

    if keyboard.pressed(KeyCode::KeyS) 
    {
        direction.y -= 1.0;
        player.1.current_direction = PlayerDirection::Down;
    }

    if keyboard.pressed(KeyCode::KeyA) 
    {
        direction.x -= 1.0;
        player.1.current_direction = PlayerDirection::Left;
    }

    if keyboard.pressed(KeyCode::KeyD) 
    {
        direction.x += 1.0;
        player.1.current_direction = PlayerDirection::Right;
    }

    if direction != Vec3::ZERO 
    {
        direction = direction.normalize_or_zero();
        player.1.state = PlayerState::Walking;

        player.0.translation.x += direction.x * time.delta_secs() * PLAYER_SPEED;
        player.0.translation.y += direction.y * time.delta_secs() * PLAYER_SPEED;
    } else 
    {
        player.1.state = PlayerState::Idle;
    }
}