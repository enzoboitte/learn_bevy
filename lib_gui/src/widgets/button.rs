use bevy::{color::Luminance, prelude::*};

use super::{TextView, impl_modifiable, text};
use crate::{
    modifiers::{Modifiable, Modifiers},
    view::{BuildCx, ParentLayout, View},
};

const DEFAULT_TINT: Color = Color::srgb(0.0, 0.48, 1.0);

/// A clickable view. Created with [`button`] or [`button_view`].
pub struct ButtonView<L> {
    label: L,
    mods: Modifiers,
}

impl_modifiable!(ButtonView<L>);

/// A text button (SwiftUI's `Button("Title") { action }`).
///
/// The action is a regular Bevy system:
/// ```ignore
/// button("Play", |mut next: ResMut<NextState<GameState>>| next.set(GameState::Playing))
/// ```
pub fn button<M>(title: impl Into<String>, action: impl IntoSystem<(), (), M> + 'static) -> ButtonView<TextView> {
    button_view(text(title).semibold(), action)
}

/// A button with any view as label (icon, image, stack...).
pub fn button_view<L: View, M>(label: L, action: impl IntoSystem<(), (), M> + 'static) -> ButtonView<L> {
    ButtonView {
        label,
        mods: Modifiers::new(Node {
            padding: UiRect::axes(px(16), px(10)),
            border_radius: BorderRadius::all(px(8)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: px(6),
            ..default()
        }),
    }
    .tint(DEFAULT_TINT)
    .on_tap(action)
}

impl<L: View> ButtonView<L> {
    /// Button color; hover and pressed colors are derived from it.
    pub fn tint(self, color: impl Into<Color>) -> Self {
        let color = color.into();
        self.background(color).hover_background(color.lighter(0.08)).pressed_background(color.darker(0.1))
    }

    /// No background, like SwiftUI's `.buttonStyle(.plain)`.
    pub fn plain(self) -> Self {
        self.tint(Color::NONE)
            .hover_background(Color::srgba(1.0, 1.0, 1.0, 0.08))
            .pressed_background(Color::srgba(1.0, 1.0, 1.0, 0.15))
    }
}

impl<L: View> View for ButtonView<L> {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let id = cx.commands.spawn(Button).id();

        let parent = std::mem::replace(&mut cx.parent, ParentLayout::Row);
        let mut children = Vec::new();
        self.label.build(cx, &mut children);
        cx.parent = parent;

        let mut entity = cx.commands.entity(id);
        self.mods.apply(parent, &mut entity);
        entity.add_children(&children);
        out.push(id);
    }
}
