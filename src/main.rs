use bevy::prelude::*;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt, config::ConfigureLoadingState};

mod entities;
mod utils;
mod world;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((
            entities::player::PlayerPlugin, 
            utils::animations::AnimationPlugin,
            world::map::MapPlugin,
        ))

        .init_state::<GameState>()

        .add_loading_state(LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Playing)
            .load_collection::<utils::game_assets::GameAssets>()
        )

        .add_systems(OnEnter(GameState::Playing), setup_camera)
        .run();
}

fn setup_camera(
    mut commands: Commands,
)
{
    commands.spawn((Camera2d, Projection::Orthographic(OrthographicProjection 
        {
            scale: 0.3,
            ..OrthographicProjection::default_2d()
        }
    )));
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
}