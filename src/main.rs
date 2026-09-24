use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt, config::ConfigureLoadingState};

use crate::{utils::{click_plugin::ClickPlugin}, world::paths::PathMap};

mod entities;
mod utils;
mod world;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_plugins((
            entities::player::PlayerPlugin, 
            utils::animations::AnimationPlugin,
            world::map::MapPlugin,
            ClickPlugin,
        ))

        .init_state::<GameState>()
        .init_resource::<PathMap>()

        .add_loading_state(LoadingState::new(GameState::Loading)
            .continue_to_state(GameState::Playing)
            .load_collection::<utils::game_assets::GameAssets>()
        )

        .add_systems(OnEnter(GameState::Playing), setup_camera)
        .add_systems(OnEnter(GameState::Playing), setup_world)
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

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawns a white plane at one unit below the origin
    commands
        .spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(128.0, 128.0))),
            MeshMaterial3d(materials.add(Color::WHITE)),
            Transform::from_translation(Vec3::ZERO),
        )
    );
}


#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
}