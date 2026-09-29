mod button;
mod controls;
mod misc;
mod stack;
mod text;
pub(crate) mod text_field;

pub use button::{ButtonView, button, button_view};
pub use controls::{
    BoundTintProgress, PickerView, ProgressView, SliderView, ToggleView, checkbox, picker, progress_bar,
    slider, toggle,
};
pub use misc::{
    DividerView, ImageView, ShapeView, SpacerView, badge, circle, divider, image, rectangle, spacer,
};
pub use stack::{Align, Stack, card, grid, hscroll_view, hstack, scroll_view, vstack, zstack};
pub use text::{TextView, dyn_text, dyn_text_world, text};
pub use text_field::{FocusedTextField, TextFieldView, text_field};

/// Implements `Modifiable` for a widget holding a `mods: Modifiers` field.
macro_rules! impl_modifiable {
    ($ty:ident $(<$($g:ident $(: $bound:path)?),+>)?) => {
        impl$(<$($g $(: $bound)?),+>)? $crate::modifiers::Modifiable for $ty$(<$($g),+>)? {
            fn modifiers(&mut self) -> &mut $crate::modifiers::Modifiers {
                &mut self.mods
            }
        }
    };
}
pub(crate) use impl_modifiable;
