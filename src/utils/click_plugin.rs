use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier3d::prelude::*;

use crate::GameState;

pub struct ClickPlugin;

impl Plugin for ClickPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EntityClicked>()
            .add_systems(Update, detect_click.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Component)]
pub struct Clickable;

#[derive(Message)]
pub struct EntityClicked {
    pub entity: Entity,
    pub click_pos: Vec2,
    pub entity_pos: Vec2,
}

fn detect_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    rapier_context: ReadRapierContext,
    clickables: Query<&GlobalTransform, With<Clickable>>,
    mut writer: MessageWriter<EntityClicked>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let (Ok(window), Ok((camera, camera_tf)), Ok(context)) =
        (windows.single(), cameras.single(), rapier_context.single())
    else {
        return;
    };

    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let Ok(click_pos) = camera.viewport_to_world_2d(camera_tf, cursor) else {
        return;
    };

    let is_clickable = |entity: Entity| clickables.contains(entity);
    let filter = QueryFilter::default().predicate(&is_clickable);

    let Some((entity, _)) =
        context.cast_ray(click_pos.extend(1000.0), Vec3::NEG_Z, 2000.0, true, filter)
    else {
        return;
    };

    let Ok(target_tf) = clickables.get(entity) else {
        return;
    };

    writer.write(EntityClicked {
        entity,
        click_pos,
        entity_pos: target_tf.translation().truncate(),
    });
}