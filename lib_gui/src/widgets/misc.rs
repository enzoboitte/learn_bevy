use bevy::prelude::*;

use super::impl_modifiable;
use crate::{
    modifiers::{Modifiable, Modifiers},
    view::{BuildCx, ParentLayout, View},
};

/// Flexible empty space that pushes its siblings apart (SwiftUI's `Spacer()`).
pub struct SpacerView {
    mods: Modifiers,
}

impl_modifiable!(SpacerView);

pub fn spacer() -> SpacerView {
    SpacerView { mods: Modifiers::new(Node { flex_grow: 1.0, ..default() }) }
}

impl View for SpacerView {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let mut entity = cx.commands.spawn_empty();
        self.mods.apply(cx.parent, &mut entity);
        out.push(entity.id());
    }
}

/// A thin line: horizontal in a `vstack`, vertical in an `hstack` (SwiftUI's `Divider()`).
pub struct DividerView {
    mods: Modifiers,
}

impl_modifiable!(DividerView);

pub fn divider() -> DividerView {
    DividerView { mods: Modifiers::new(Node { flex_shrink: 0.0, ..default() }) }
        .background(Color::srgba(1.0, 1.0, 1.0, 0.2))
}

impl View for DividerView {
    fn build(mut self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let node = &mut self.mods.node;
        if node.align_self == AlignSelf::Auto {
            node.align_self = AlignSelf::Stretch;
        }
        match cx.parent {
            ParentLayout::Row => node.min_width = px(1),
            _ => node.min_height = px(1),
        }
        let mut entity = cx.commands.spawn_empty();
        self.mods.apply(cx.parent, &mut entity);
        out.push(entity.id());
    }
}

/// An image (SwiftUI's `Image(...)`). Use `.frame(w, h)` to size it.
pub struct ImageView {
    image: ImageNode,
    mods: Modifiers,
}

impl_modifiable!(ImageView);

pub fn image(handle: Handle<Image>) -> ImageView {
    ImageView { image: ImageNode::new(handle), mods: Modifiers::new(Node::default()) }
}

impl ImageView {
    /// Multiplies the image colors (white = unchanged).
    pub fn tint(mut self, color: impl Into<Color>) -> Self {
        self.image.color = color.into();
        self
    }
    pub fn flip_x(mut self) -> Self {
        self.image.flip_x = true;
        self
    }
    pub fn flip_y(mut self) -> Self {
        self.image.flip_y = true;
        self
    }
    /// Uses a sprite from a texture atlas.
    pub fn atlas(mut self, layout: Handle<TextureAtlasLayout>, index: usize) -> Self {
        self.image.texture_atlas = Some(TextureAtlas { layout, index });
        self
    }
    /// Shows only a region of the image, in pixels (for sprite sheets with irregular layouts).
    ///
    /// ```ignore
    /// image(ui.clone()).region(455.0, 7.0, 18.0, 19.0).frame(54.0, 57.0)
    /// ```
    pub fn region(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.image.rect = Some(Rect::new(x, y, x + width, y + height));
        self
    }
    /// Plays atlas `frames` at `fps` in a loop (use `.atlas(..)` first).
    pub fn animate_frames(self, frames: impl Into<Vec<usize>>, fps: f32) -> Self {
        self.insert(crate::anim::FrameAnimation::new(frames.into(), fps, true))
    }
    /// Plays atlas `frames` once and stays on the last one.
    pub fn animate_frames_once(self, frames: impl Into<Vec<usize>>, fps: f32) -> Self {
        self.insert(crate::anim::FrameAnimation::new(frames.into(), fps, false))
    }
    /// Ignores the image's aspect ratio and stretches it to the frame.
    pub fn stretch(mut self) -> Self {
        self.image.image_mode = NodeImageMode::Stretch;
        self
    }
}

impl View for ImageView {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let mut entity = cx.commands.spawn(self.image);
        self.mods.apply(cx.parent, &mut entity);
        out.push(entity.id());
    }
}

/// A plain colored shape (SwiftUI's `Rectangle()` / `Circle()`).
pub struct ShapeView {
    mods: Modifiers,
}

impl_modifiable!(ShapeView);

/// A colored rectangle; size it with `.frame(..)`, `.fill_width()`, `.grow()`...
pub fn rectangle(color: impl Into<Color>) -> ShapeView {
    ShapeView { mods: Modifiers::new(Node::default()) }.background(color)
}

/// A colored circle.
pub fn circle(diameter: f32, color: impl Into<Color>) -> ShapeView {
    rectangle(color).frame(diameter, diameter).capsule().no_shrink()
}

impl View for ShapeView {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let mut entity = cx.commands.spawn_empty();
        self.mods.apply(cx.parent, &mut entity);
        out.push(entity.id());
    }
}

/// A small pill with a label (notifications count, "NEW"...).
pub fn badge(label: impl Into<String>) -> super::Stack<super::TextView> {
    super::hstack(super::text(label).caption().bold())
        .padding_xy(8.0, 2.0)
        .capsule()
        .background(Color::srgb(0.9, 0.2, 0.25))
}
