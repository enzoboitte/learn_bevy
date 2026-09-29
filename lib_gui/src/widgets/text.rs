use bevy::prelude::*;

use super::impl_modifiable;
use crate::{
    binding::{BoundText, BoundTextColor},
    font::UsesGuiFont,
    modifiers::Modifiers,
    view::{BuildCx, View},
};

/// A label. Created with [`text`], [`dyn_text`] or [`dyn_text_world`].
pub struct TextView {
    content: String,
    bound: Option<BoundText>,
    font: TextFont,
    custom_font: bool,
    color: Color,
    bound_color: Option<BoundTextColor>,
    layout: TextLayout,
    shadow: Option<TextShadow>,
    mods: Modifiers,
}

impl_modifiable!(TextView);

/// Static text (SwiftUI's `Text("...")`).
pub fn text(content: impl Into<String>) -> TextView {
    TextView {
        content: content.into(),
        bound: None,
        font: TextFont::from_font_size(16.0),
        custom_font: false,
        color: Color::WHITE,
        bound_color: None,
        layout: TextLayout::default(),
        shadow: None,
        mods: Modifiers::new(Node::default()),
    }
}

/// Text computed from resource `R`, updated automatically when it changes.
///
/// ```ignore
/// dyn_text(|score: &Score| format!("Score: {}", score.0))
/// ```
pub fn dyn_text<R: Resource>(f: impl Fn(&R) -> String + Send + Sync + 'static) -> TextView {
    let mut view = text("");
    view.bound = Some(BoundText(Box::new(move |world| world.get_resource::<R>().map(&f))));
    view
}

/// Text computed from the whole `World` (several resources, queries...).
pub fn dyn_text_world(f: impl Fn(&World) -> String + Send + Sync + 'static) -> TextView {
    let mut view = text("");
    view.bound = Some(BoundText(Box::new(move |world| Some(f(world)))));
    view
}

impl TextView {
    pub fn font_size(mut self, size: f32) -> Self {
        self.font.font_size = size.into();
        self
    }
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }
    pub fn font(mut self, font: Handle<Font>) -> Self {
        self.font = self.font.with_font(font);
        self.custom_font = true;
        self
    }
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.font.weight = weight;
        self
    }
    pub fn bold(self) -> Self {
        self.weight(FontWeight::BOLD)
    }
    pub fn semibold(self) -> Self {
        self.weight(FontWeight::SEMIBOLD)
    }
    pub fn italic(mut self) -> Self {
        self.font.style = FontStyle::Italic;
        self
    }
    /// Alignment of the lines of a multi-line text.
    pub fn justify(mut self, justify: Justify) -> Self {
        self.layout.justify = justify;
        self
    }
    /// Centers the lines of a multi-line text.
    pub fn multiline_center(self) -> Self {
        self.justify(Justify::Center)
    }
    /// Keeps the text on a single line.
    pub fn no_wrap(mut self) -> Self {
        self.layout.linebreak = LineBreak::NoWrap;
        self
    }
    /// Drop shadow behind the glyphs.
    pub fn text_shadow(mut self, color: impl Into<Color>, x: f32, y: f32) -> Self {
        self.shadow = Some(TextShadow { offset: Vec2::new(x, y), color: color.into() });
        self
    }
    /// Text color computed from resource `R`, updated automatically.
    pub fn bind_color<R: Resource>(mut self, f: impl Fn(&R) -> Color + Send + Sync + 'static) -> Self {
        self.bound_color = Some(BoundTextColor(Box::new(move |world| world.get_resource::<R>().map(&f))));
        self
    }

    // SwiftUI-like text styles.
    pub fn large_title(self) -> Self {
        self.font_size(40.0).bold()
    }
    pub fn title(self) -> Self {
        self.font_size(30.0).bold()
    }
    pub fn headline(self) -> Self {
        self.font_size(20.0).semibold()
    }
    pub fn body(self) -> Self {
        self.font_size(16.0)
    }
    pub fn caption(self) -> Self {
        self.font_size(12.0)
    }
}

impl View for TextView {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let mut entity =
            cx.commands.spawn((Text::new(self.content), self.font, TextColor(self.color), self.layout));
        if !self.custom_font {
            entity.insert(UsesGuiFont);
        }
        if let Some(bound) = self.bound {
            entity.insert(bound);
        }
        if let Some(bound) = self.bound_color {
            entity.insert(bound);
        }
        if let Some(shadow) = self.shadow {
            entity.insert(shadow);
        }
        self.mods.apply(cx.parent, &mut entity);
        out.push(entity.id());
    }
}
