use bevy::prelude::*;

/// Font used by every `text`, `button`, `text_field`... that has no explicit `.font(..)`.
///
/// Bevy's built-in font has no accented characters (é, à, ç...): load your own to display them.
/// ```ignore
/// commands.insert_resource(GuiFont(assets.load("fonts/Inter.ttf")));
/// ```
#[derive(Resource, Clone, Debug)]
pub struct GuiFont(pub Handle<Font>);

/// Marks texts that follow [`GuiFont`].
#[derive(Component, Default)]
pub(crate) struct UsesGuiFont;

pub(crate) fn apply_gui_font(font: Option<Res<GuiFont>>, mut texts: Query<(Ref<UsesGuiFont>, &mut TextFont)>) {
    let Some(font) = font else { return };
    for (marker, mut text_font) in &mut texts {
        if font.is_changed() || marker.is_added() {
            text_font.font = FontSource::Handle(font.0.clone());
        }
    }
}
