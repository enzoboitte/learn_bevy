use std::sync::Arc;

use bevy::{
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
    },
    prelude::*,
};

use super::controls::ACCENT;
use crate::{
    font::UsesGuiFont,
    MutResource,
    modifiers::{Modifiable, Modifiers},
    view::{BuildCx, View},
};

/// The text field currently receiving keyboard input, if any.
#[derive(Resource, Default, Debug)]
pub struct FocusedTextField(pub Option<Entity>);

#[derive(Component)]
pub(crate) struct TextFieldState {
    value: String,
    placeholder: String,
    dirty: bool,
    max_length: Option<usize>,
    password: bool,
    label: Entity,
    border: Color,
    focus_border: Color,
}

#[derive(Component)]
pub(crate) struct TextFieldSync {
    get: Box<dyn Fn(&World) -> Option<String> + Send + Sync>,
    set: Arc<dyn Fn(&mut World, String) + Send + Sync>,
}

#[derive(Component)]
pub(crate) struct TextFieldLabel;

/// A one-line text input. Created with [`text_field`].
pub struct TextFieldView<R: MutResource> {
    placeholder: String,
    get: Arc<dyn Fn(&R) -> String + Send + Sync>,
    set: Arc<dyn Fn(&mut R, String) + Send + Sync>,
    max_length: Option<usize>,
    password: bool,
    font_size: f32,
    border: Color,
    focus_border: Color,
    mods: Modifiers,
}

impl<R: MutResource> Modifiable for TextFieldView<R> {
    fn modifiers(&mut self) -> &mut Modifiers {
        &mut self.mods
    }
}

/// A text input bound to a `String` of resource `R` (SwiftUI's `TextField`).
///
/// Click to focus, type, `Enter` / `Escape` or click elsewhere to leave.
/// ```ignore
/// text_field("Player name", |p: &Profile| p.name.clone(), |p, name| p.name = name)
/// ```
pub fn text_field<R: MutResource>(
    placeholder: impl Into<String>,
    get: impl Fn(&R) -> String + Send + Sync + 'static,
    set: impl Fn(&mut R, String) + Send + Sync + 'static,
) -> TextFieldView<R> {
    TextFieldView {
        placeholder: placeholder.into(),
        get: Arc::new(get),
        set: Arc::new(set),
        max_length: None,
        password: false,
        font_size: 16.0,
        border: Color::srgb(0.3, 0.3, 0.34),
        focus_border: ACCENT,
        mods: Modifiers::new(Node {
            min_width: px(200),
            padding: UiRect::axes(px(10), px(7)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(6)),
            align_items: AlignItems::Center,
            ..default()
        }),
    }
    .background(Color::srgb(0.1, 0.1, 0.12))
}

impl<R: MutResource> TextFieldView<R> {
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }
    /// Hides the characters.
    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
    pub fn font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    /// Border color while focused.
    pub fn tint(mut self, color: impl Into<Color>) -> Self {
        self.focus_border = color.into();
        self
    }
    /// Border color while not focused.
    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        self.border = color.into();
        self
    }
}

impl<R: MutResource> View for TextFieldView<R> {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let label = cx
            .commands
            .spawn((
                Text::new(""),
                TextFont::from_font_size(self.font_size),
                TextColor(Color::WHITE),
                TextLayout::no_wrap(),
                TextFieldLabel,
                UsesGuiFont,
                Pickable::IGNORE,
            ))
            .id();

        let (get, set) = (self.get, self.set);
        let sync = TextFieldSync {
            get: Box::new(move |world| world.get_resource::<R>().map(|r| get(r))),
            set: Arc::new(move |world, value| {
                if let Some(mut res) = world.get_resource_mut::<R>() {
                    set(&mut res, value);
                }
            }),
        };
        let state = TextFieldState {
            value: String::new(),
            placeholder: self.placeholder,
            dirty: false,
            max_length: self.max_length,
            password: self.password,
            label,
            border: self.border,
            focus_border: self.focus_border,
        };

        let mut entity = cx.commands.spawn((state, sync, BorderColor::all(self.border), Interaction::default()));
        let id = entity.id();
        self.mods.apply(cx.parent, &mut entity);
        entity.add_child(label).observe(move |mut click: On<Pointer<Click>>, mut focus: ResMut<FocusedTextField>| {
            focus.0 = Some(id);
            click.propagate(false);
        });
        out.push(id);
    }
}

/// Clicking anything that is not a text field removes the focus.
pub(crate) fn unfocus_on_click_elsewhere(
    click: On<Pointer<Click>>,
    mut focus: ResMut<FocusedTextField>,
    fields: Query<(), With<TextFieldState>>,
) {
    if click.entity == click.original_event_target() && !fields.contains(click.entity) && focus.0.is_some() {
        focus.0 = None;
    }
}

pub(crate) fn text_field_keyboard(
    mut keys: MessageReader<KeyboardInput>,
    mut focus: ResMut<FocusedTextField>,
    mut fields: Query<&mut TextFieldState>,
) {
    let Some(entity) = focus.0 else {
        keys.clear();
        return;
    };
    let Ok(mut field) = fields.get_mut(entity) else {
        focus.0 = None;
        return;
    };

    for key in keys.read() {
        if key.state != ButtonState::Pressed {
            continue;
        }
        match &key.logical_key {
            Key::Backspace => {
                field.value.pop();
                field.dirty = true;
            }
            Key::Enter | Key::Escape => focus.0 = None,
            Key::Space => push_chars(&mut field, " "),
            Key::Character(chars) => push_chars(&mut field, chars),
            _ => {}
        }
    }
}

fn push_chars(field: &mut TextFieldState, chars: &str) {
    for c in chars.chars().filter(|c| !c.is_control()) {
        if field.max_length.is_some_and(|max| field.value.chars().count() >= max) {
            break;
        }
        field.value.push(c);
        field.dirty = true;
    }
}

/// Two-way sync between the fields and their resources.
pub(crate) fn sync_text_fields(world: &mut World, query: &mut QueryState<(Entity, &TextFieldState, &TextFieldSync)>) {
    let mut pushes = Vec::new();
    let mut pulls = Vec::new();
    for (entity, state, sync) in query.iter(world) {
        if state.dirty {
            pushes.push((entity, sync.set.clone(), state.value.clone()));
        } else if let Some(value) = (sync.get)(world)
            && value != state.value
        {
            pulls.push((entity, value));
        }
    }

    for (entity, set, value) in pushes {
        set(world, value);
        if let Some(mut state) = world.get_mut::<TextFieldState>(entity) {
            state.dirty = false;
        }
    }
    for (entity, value) in pulls {
        if let Some(mut state) = world.get_mut::<TextFieldState>(entity) {
            state.value = value;
        }
    }
}

pub(crate) fn update_text_field_display(
    focus: Res<FocusedTextField>,
    mut fields: Query<(Entity, &TextFieldState, &mut BorderColor)>,
    mut labels: Query<(&mut Text, &mut TextColor), With<TextFieldLabel>>,
) {
    for (entity, state, mut border) in &mut fields {
        let focused = focus.0 == Some(entity);

        let wanted_border = BorderColor::all(if focused { state.focus_border } else { state.border });
        if *border != wanted_border {
            *border = wanted_border;
        }

        let Ok((mut text, mut color)) = labels.get_mut(state.label) else { continue };
        let (content, wanted_color) = if state.value.is_empty() && !focused {
            (state.placeholder.clone(), Color::srgb(0.5, 0.5, 0.55))
        } else {
            let shown = if state.password { "*".repeat(state.value.chars().count()) } else { state.value.clone() };
            (if focused { format!("{shown}|") } else { shown }, Color::WHITE)
        };
        if text.0 != content {
            text.0 = content;
        }
        if color.0 != wanted_color {
            color.0 = wanted_color;
        }
    }
}
