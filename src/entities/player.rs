use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::GameState;
use crate::utils::click_plugin::{CursorEvent, EntityClicked};
use crate::utils::game_assets::GameAssets;
use crate::utils::animations::*;
use crate::world::map::tile_to_world;
use crate::world::paths::{PATH_TILE_BY_MASK, PathMap, PathTile, world_to_tile};

const PLAYER_SPEED: f32 = 100.0;
const GRID_CELL_SIZE: f32 = 16.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin 
{
    fn build(&self, app: &mut App) 
    {
        app
            .add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(Update, (move_player, update_indices, on_entity_clicked_over)
                                                        .run_if(in_state(GameState::Playing)));
    }
}

pub enum PlayerDirection 
{
    Up,
    Down,
    Left,
    Right
}
pub enum PlayerState 
{
    Idle,
    Walking
}

#[derive(Component)]
pub struct InteractionRange(pub f32);

#[derive(Component)]
pub struct Player 
{
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
        Transform::from_xyz(0.0, 0.0, 2.0),
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
        
        Velocity::zero(),
        RigidBody::Dynamic,
        GravityScale(0.0),
        Collider::cuboid(16.0 / 2.0, 16.0 / 2.0, 0.1),
        LockedAxes::ROTATION_LOCKED | LockedAxes::TRANSLATION_LOCKED_Z,
        InteractionRange(22.0),
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
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player: Single<(&mut Transform, &mut Player, &mut Velocity), With<Player>>,
)
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

        //player.0.translation.x += direction.x * time.delta_secs() * PLAYER_SPEED;
        //player.0.translation.y += direction.y * time.delta_secs() * PLAYER_SPEED;
        player.2.linear.x = direction.x * PLAYER_SPEED;
        player.2.linear.y = direction.y * PLAYER_SPEED;
    } else 
    {
        player.1.state = PlayerState::Idle;
        player.2.linear = Vec3::ZERO;
    }
}

fn on_entity_clicked_over(
    mut commands: Commands,
    mut reader: MessageReader<EntityClicked>,
    player: Single<(&GlobalTransform, &InteractionRange), With<Player>>,

    mut gizmos: Gizmos,
    game_assets: Res<GameAssets>,
    mut path_map: ResMut<PathMap>,
    mut path_tiles: Query<(&PathTile, &mut Sprite)>,
) 
{
    let (player_tf, range) = *player;
    let player_pos = player_tf.translation().truncate();

    for event in reader.read() 
    {
        let distance = player_pos.distance(event.entity_pos);
        let pos = Vec2::new(
            (event.mouse_pos.x / GRID_CELL_SIZE).floor() * GRID_CELL_SIZE + GRID_CELL_SIZE / 2.0,
            (event.mouse_pos.y / GRID_CELL_SIZE).floor() * GRID_CELL_SIZE + GRID_CELL_SIZE / 2.0,
        );

        if distance > range.0
        { continue; }

        gizmos.rect_2d(
            Isometry2d::from_translation(pos),
            Vec2::splat(GRID_CELL_SIZE),
            if distance > range.0 { Color::srgb(1.0, 0.2, 0.2) } else { Color::srgb(0.2, 1.0, 0.2) },
        );

        if event.cursor_event == CursorEvent::CLICK
        {
            if let Some(entity) = event.entity 
            {
                commands.entity(entity).despawn();
            } else if let Some((x, y)) = world_to_tile(pos)
            {
                path_map.set_path(x, y);

                if let Some(tile_index) = path_map.tile_index(x, y)
                {
                    let position = tile_to_world(x, y);
                    let already_exists = path_tiles
                        .iter()
                        .any(|(tile, _)| tile.x == x && tile.y == y);

                    if !already_exists
                    {
                        commands.spawn((
                            Sprite::from_atlas_image(
                                game_assets.paths_texture.clone(), 
                                TextureAtlas { 
                                    layout: game_assets.paths_layout.clone(),
                                    index: tile_index,
                                }
                            ),
                            Transform::from_xyz(position.x, position.y, 1.0),
                            PathTile { x, y },
                        ));
                    }

                    for (tile_x, tile_y) in PathMap::affected_tiles(x, y) 
                    {
                        if PathMap::is_inside(tile_x, tile_y) 
                        {
                            let mask = path_map.path_mask(tile_x, tile_y);
                            let index = PATH_TILE_BY_MASK[mask as usize];

                            for (tile, mut sprite) in path_tiles.iter_mut() 
                            {
                                if tile.x == tile_x && tile.y == tile_y 
                                {
                                    if let Some(atlas) = &mut sprite.texture_atlas 
                                    {
                                        atlas.index = index;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}