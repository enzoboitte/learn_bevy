use bevy::prelude::*;

use super::impl_modifiable;
use crate::{
    modifiers::{Modifiable, Modifiers},
    view::{BuildCx, ParentLayout, View},
};

/// Cross-axis alignment of a stack's children
/// (`Start` = leading in a `vstack`, top in an `hstack`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Align {
    Start,
    #[default]
    Center,
    End,
    Stretch,
}

impl From<Align> for AlignItems {
    fn from(align: Align) -> Self {
        match align {
            Align::Start => AlignItems::Start,
            Align::Center => AlignItems::Center,
            Align::End => AlignItems::End,
            Align::Stretch => AlignItems::Stretch,
        }
    }
}

impl From<Align> for JustifyItems {
    fn from(align: Align) -> Self {
        match align {
            Align::Start => JustifyItems::Start,
            Align::Center => JustifyItems::Center,
            Align::End => JustifyItems::End,
            Align::Stretch => JustifyItems::Stretch,
        }
    }
}

/// `vstack`, `hstack` or `zstack`.
pub struct Stack<C> {
    pub(crate) layout: ParentLayout,
    pub(crate) content: C,
    pub(crate) mods: Modifiers,
}

impl_modifiable!(Stack<C>);

/// Children laid out top to bottom (SwiftUI's `VStack`).
pub fn vstack<C: View>(content: C) -> Stack<C> {
    Stack::new(ParentLayout::Column, content, FlexDirection::Column)
}

/// Children laid out left to right (SwiftUI's `HStack`).
pub fn hstack<C: View>(content: C) -> Stack<C> {
    Stack::new(ParentLayout::Row, content, FlexDirection::Row)
}

/// Children stacked on top of each other, the last one in front (SwiftUI's `ZStack`).
pub fn zstack<C: View>(content: C) -> Stack<C> {
    let mut stack = Stack::new(ParentLayout::Overlay, content, FlexDirection::Row);
    stack.mods.node.display = Display::Grid;
    stack.mods.node.justify_items = JustifyItems::Center;
    stack
}

/// Vertical stack that scrolls with the mouse wheel when its content is taller than it.
///
/// Give it a height (`.height(px(300.))`, `.max_height(300.)` or `.grow()`).
pub fn scroll_view<C: View>(content: C) -> Stack<C> {
    vstack(content)
        .align(Align::Stretch)
        .node(|n| n.overflow = Overflow::scroll_y())
        .insert(ScrollPosition::default())
        .observe(scroll_observer)
}

/// Horizontal version of [`scroll_view`].
pub fn hscroll_view<C: View>(content: C) -> Stack<C> {
    hstack(content)
        .node(|n| n.overflow = Overflow::scroll_x())
        .insert(ScrollPosition::default())
        .observe(scroll_observer)
}

fn scroll_observer(mut scroll: On<Pointer<Scroll>>, mut positions: Query<(&mut ScrollPosition, &Node)>) {
    let Ok((mut position, node)) = positions.get_mut(scroll.entity) else { return };
    let factor = match scroll.unit {
        bevy::input::mouse::MouseScrollUnit::Line => 24.0,
        bevy::input::mouse::MouseScrollUnit::Pixel => 1.0,
    };
    let (dx, dy) = (scroll.x * factor, scroll.y * factor);
    if node.overflow.y == OverflowAxis::Scroll {
        position.y = (position.y - dy).max(0.0);
    }
    if node.overflow.x == OverflowAxis::Scroll {
        // Vertical wheels also scroll horizontal views.
        let delta = if dx != 0.0 { dx } else { dy };
        position.x = (position.x - delta).max(0.0);
    }
    scroll.propagate(false);
}

/// Grid with `columns` equal columns, filled row by row (SwiftUI's `LazyVGrid`).
pub fn grid<C: View>(columns: u16, content: C) -> Stack<C> {
    let mut stack = Stack::new(ParentLayout::Root, content, FlexDirection::Row);
    stack.mods.node.display = Display::Grid;
    stack.mods.node.grid_template_columns = vec![RepeatedGridTrack::flex(columns, 1.0)];
    stack.mods.node.align_items = AlignItems::Stretch;
    stack
}

/// A rounded panel with padding, background and shadow, children aligned to the start.
pub fn card<C: View>(content: C) -> Stack<C> {
    vstack(content)
        .align(Align::Start)
        .padding(16.0)
        .background(Color::srgb(0.13, 0.13, 0.16))
        .corner_radius(12.0)
        .shadow(Color::srgba(0.0, 0.0, 0.0, 0.4), 12.0, 0.0, 4.0)
}

impl<C: View> Stack<C> {
    pub(crate) fn new(layout: ParentLayout, content: C, flex_direction: FlexDirection) -> Self {
        let mods = Modifiers::new(Node {
            flex_direction,
            align_items: AlignItems::Center,
            row_gap: px(8),
            column_gap: px(8),
            ..default()
        });
        Self { layout, content, mods }
    }

    /// Space between children (default: 8px).
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.mods.node.row_gap = px(spacing);
        self.mods.node.column_gap = px(spacing);
        self
    }

    /// Cross-axis alignment of the children (default: center).
    pub fn align(mut self, align: Align) -> Self {
        self.mods.node.align_items = align.into();
        if self.layout == ParentLayout::Overlay {
            self.mods.node.justify_items = align.into();
        }
        self
    }

    /// Main-axis distribution of the children (`JustifyContent::SpaceBetween`, ...).
    pub fn justify(mut self, justify: JustifyContent) -> Self {
        self.mods.node.justify_content = justify;
        self
    }

    /// Centers the children on both axes.
    pub fn center(mut self) -> Self {
        self.mods.node.justify_content = JustifyContent::Center;
        self.mods.node.align_items = AlignItems::Center;
        self.mods.node.justify_items = JustifyItems::Center;
        self
    }

    /// Wraps children onto several lines when they don't fit.
    pub fn wrap(mut self) -> Self {
        self.mods.node.flex_wrap = FlexWrap::Wrap;
        self
    }
}

impl<C: View> View for Stack<C> {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let id = cx.commands.spawn_empty().id();

        let parent = std::mem::replace(&mut cx.parent, self.layout);
        let mut children = Vec::new();
        self.content.build(cx, &mut children);
        cx.parent = parent;

        let mut entity = cx.commands.entity(id);
        self.mods.apply(parent, &mut entity);
        entity.add_children(&children);
        out.push(id);
    }
}
