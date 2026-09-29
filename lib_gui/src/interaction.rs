use bevy::{
    ecs::{lifecycle::HookContext, system::SystemId, world::DeferredWorld},
    prelude::*,
};

/// Background colors swapped according to [`Interaction`].
#[derive(Component, Clone, Copy, Debug)]
pub struct InteractiveColors {
    pub normal: Color,
    pub hovered: Color,
    pub pressed: Color,
}

pub(crate) fn update_interactive_colors(
    mut query: Query<(&Interaction, &InteractiveColors, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, colors, mut background) in &mut query {
        background.0 = match interaction {
            Interaction::Pressed => colors.pressed,
            Interaction::Hovered => colors.hovered,
            Interaction::None => colors.normal,
        };
    }
}

/// One-shot systems registered for a view's actions; unregistered when the view is despawned.
#[derive(Component)]
#[component(on_remove = unregister_owned_systems)]
pub struct OwnedSystems(pub(crate) Vec<SystemId>);

fn unregister_owned_systems(mut world: DeferredWorld, ctx: HookContext) {
    let Some(owned) = world.get::<OwnedSystems>(ctx.entity) else { return };
    let ids = owned.0.clone();
    let mut commands = world.commands();
    for id in ids {
        commands.unregister_system(id);
    }
}
