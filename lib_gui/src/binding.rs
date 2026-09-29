//! Reactive views: values recomputed from the `World` every frame and applied
//! only when they change.

use bevy::prelude::*;

type Compute<T> = Box<dyn Fn(&World) -> T + Send + Sync>;

/// Text recomputed every frame (see [`crate::widgets::dyn_text`]).
#[derive(Component)]
pub struct BoundText(pub(crate) Compute<Option<String>>);

/// Text color recomputed every frame (see [`crate::widgets::TextView::bind_color`]).
#[derive(Component)]
pub struct BoundTextColor(pub(crate) Compute<Option<Color>>);

/// Background recomputed every frame (see [`crate::modifiers::Modifiable::bind_background`]).
#[derive(Component)]
pub struct BoundBackground(pub(crate) Compute<Option<Color>>);

/// Node edited every frame (see [`crate::modifiers::Modifiable::bind_node`]).
#[derive(Component)]
pub struct BoundNode(pub(crate) Box<dyn Fn(&World, &mut Node) + Send + Sync>);

/// Display toggled every frame (see [`crate::modifiers::Modifiable::show_if`]).
#[derive(Component)]
pub struct BoundDisplay {
    condition: Compute<bool>,
    shown_display: Option<Display>,
}

impl BoundDisplay {
    pub fn new(condition: impl Fn(&World) -> bool + Send + Sync + 'static) -> Self {
        Self { condition: Box::new(condition), shown_display: None }
    }
}

pub(crate) fn update_bound_text(world: &mut World, query: &mut QueryState<(Entity, &BoundText, &Text)>) {
    let changes: Vec<(Entity, String)> = query
        .iter(world)
        .filter_map(|(entity, bound, text)| {
            let value = (bound.0)(world)?;
            (value != text.0).then_some((entity, value))
        })
        .collect();

    for (entity, value) in changes {
        if let Some(mut text) = world.get_mut::<Text>(entity) {
            text.0 = value;
        }
    }
}

pub(crate) fn update_bound_text_color(
    world: &mut World,
    query: &mut QueryState<(Entity, &BoundTextColor, &TextColor)>,
) {
    let changes: Vec<(Entity, Color)> = query
        .iter(world)
        .filter_map(|(entity, bound, color)| {
            let value = (bound.0)(world)?;
            (value != color.0).then_some((entity, value))
        })
        .collect();

    for (entity, value) in changes {
        if let Some(mut color) = world.get_mut::<TextColor>(entity) {
            color.0 = value;
        }
    }
}

pub(crate) fn update_bound_background(
    world: &mut World,
    query: &mut QueryState<(Entity, &BoundBackground, Option<&BackgroundColor>)>,
) {
    let changes: Vec<(Entity, Color)> = query
        .iter(world)
        .filter_map(|(entity, bound, background)| {
            let value = (bound.0)(world)?;
            (Some(value) != background.map(|b| b.0)).then_some((entity, value))
        })
        .collect();

    for (entity, value) in changes {
        if let Ok(mut entity) = world.get_entity_mut(entity) {
            entity.insert(BackgroundColor(value));
        }
    }
}

pub(crate) fn update_bound_node(world: &mut World, query: &mut QueryState<(Entity, &BoundNode, &Node)>) {
    let changes: Vec<(Entity, Node)> = query
        .iter(world)
        .filter_map(|(entity, bound, node)| {
            let mut value = node.clone();
            (bound.0)(world, &mut value);
            (value != *node).then_some((entity, value))
        })
        .collect();

    for (entity, value) in changes {
        if let Some(mut node) = world.get_mut::<Node>(entity) {
            *node = value;
        }
    }
}

pub(crate) fn update_bound_display(world: &mut World, query: &mut QueryState<(Entity, &BoundDisplay, &Node)>) {
    let changes: Vec<(Entity, bool)> = query
        .iter(world)
        .filter_map(|(entity, bound, node)| {
            let show = (bound.condition)(world);
            let shown = node.display != Display::None;
            (show != shown).then_some((entity, show))
        })
        .collect();

    for (entity, show) in changes {
        let Ok(mut entity) = world.get_entity_mut(entity) else { continue };
        let current = entity.get::<Node>().map(|n| n.display).unwrap_or_default();
        let remembered = {
            let mut bound = entity.get_mut::<BoundDisplay>().unwrap();
            if !show {
                bound.shown_display = Some(current);
            }
            bound.shown_display.unwrap_or_default()
        };
        if let Some(mut node) = entity.get_mut::<Node>() {
            node.display = if show { remembered } else { Display::None };
        }
    }
}
