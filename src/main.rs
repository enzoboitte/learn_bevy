use bevy::prelude::*;

mod entities;
mod utils;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins((entities::player::PlayerPlugin, utils::animations::AnimationPlugin))
        .add_systems(Startup, setup_camera)
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