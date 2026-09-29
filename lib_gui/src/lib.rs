//! # lib_gui
//!
//! Build Bevy UIs declaratively, SwiftUI style.
//!
//! ```ignore
//! use bevy::prelude::*;
//! use lib_gui::prelude::*;
//!
//! fn setup(mut commands: Commands) {
//!     commands.spawn(Camera2d);
//!     commands.spawn_view(
//!         vstack((
//!             text("Hello Bevy").title(),
//!             dyn_text(|score: &Score| format!("Score: {}", score.0)),
//!             button("+1", |mut score: ResMut<Score>| score.0 += 1),
//!         ))
//!         .spacing(12.)
//!         .fill()
//!         .center(),
//!     );
//! }
//! ```

#![allow(clippy::type_complexity)]


use bevy::{ecs::component::Mutable, prelude::*};

pub mod binding;
pub mod font;
pub mod interaction;
pub mod modifiers;
pub mod view;
pub mod widgets;

/// A resource that can be modified (every `#[derive(Resource)]` type by default).
/// Required by the controls that write to a resource (toggle, slider, text_field...).
pub trait MutResource: Resource<Mutability = Mutable> {}
impl<T: Resource<Mutability = Mutable>> MutResource for T {}

pub mod prelude {
    pub use crate::font::GuiFont;
    pub use crate::{GuiPlugin, MutResource};
    pub use crate::modifiers::Modifiable;
    pub use crate::view::{AnyView, SpawnViewExt, View, for_each};
    pub use crate::widgets::{
        Align, FocusedTextField, badge, button, button_view, card, checkbox, circle, divider,
        dyn_text, dyn_text_world, grid, hscroll_view, hstack, image, picker, progress_bar,
        rectangle, scroll_view, slider, spacer, text, text_field, toggle, vstack, zstack,
    };
}

/// Add it to your app: `app.add_plugins(GuiPlugin)`.
pub struct GuiPlugin;

impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        use widgets::text_field;

        app.init_resource::<text_field::FocusedTextField>()
            .add_observer(text_field::unfocus_on_click_elsewhere)
            .add_systems(
                Update,
                (
                    interaction::update_interactive_colors,
                    font::apply_gui_font,
                    (text_field::text_field_keyboard, text_field::sync_text_fields).chain(),
                    (
                        binding::update_bound_text,
                        binding::update_bound_text_color,
                        binding::update_bound_background,
                        binding::update_bound_node,
                        binding::update_bound_display,
                        text_field::update_text_field_display,
                    ),
                )
                    .chain(),
            );
    }
}
