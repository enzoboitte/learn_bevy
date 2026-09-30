//! Small procedural animations driven by modifiers (`.hover_scale`, `.pulse`, `.bob`...).
//!
//! They only touch `UiTransform`, so they never move the other views around.

use std::f32::consts::{PI, TAU};

use bevy::prelude::*;

/// Animation settings of a view. Built by the animation modifiers of [`crate::modifiers::Modifiable`].
#[derive(Component, Clone, Debug)]
pub struct UiAnimation {
    pub hover_scale: f32,
    pub press_scale: f32,
    /// (amplitude, cycles per second)
    pub pulse: Option<(f32, f32)>,
    /// (pixels, cycles per second)
    pub bob: Option<(f32, f32)>,
    /// (degrees, cycles per second)
    pub wiggle: Option<(f32, f32)>,
    /// Degrees per second.
    pub spin: f32,
    /// Duration of the pop-in when the view appears.
    pub pop_in: Option<f32>,
    pub tap_bounce: bool,
    pub(crate) base: UiTransform,
    interact: f32,
    bounce: f32,
    elapsed: f32,
}

impl Default for UiAnimation {
    fn default() -> Self {
        Self {
            hover_scale: 1.0,
            press_scale: 1.0,
            pulse: None,
            bob: None,
            wiggle: None,
            spin: 0.0,
            pop_in: None,
            tap_bounce: false,
            base: UiTransform::IDENTITY,
            interact: 1.0,
            bounce: 0.0,
            elapsed: 0.0,
        }
    }
}

impl UiAnimation {
    pub(crate) fn needs_interaction(&self) -> bool {
        self.hover_scale != 1.0 || self.press_scale != 1.0
    }

    /// Restarts the tap bounce (also triggered automatically on click with `.tap_bounce()`).
    pub fn bounce(&mut self) {
        self.bounce = 1.0;
    }

    /// Restarts the pop-in animation.
    pub fn replay(&mut self) {
        self.elapsed = 0.0;
    }
}

fn ease_out_back(x: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
}

fn px_of(val: Val) -> f32 {
    match val {
        Val::Px(v) => v,
        _ => 0.0,
    }
}

pub(crate) fn bounce_on_click(click: On<Pointer<Click>>, mut animations: Query<&mut UiAnimation>) {
    if let Ok(mut animation) = animations.get_mut(click.entity) {
        animation.bounce();
    }
}

pub(crate) fn animate_ui(
    time: Res<Time>,
    mut query: Query<(&mut UiAnimation, &mut UiTransform, Option<&Interaction>)>,
) {
    let dt = time.delta_secs();
    for (mut anim, mut transform, interaction) in &mut query {
        anim.elapsed += dt;
        let t = anim.elapsed;

        // Smoothly follow the hover / press target.
        let target = match interaction {
            Some(Interaction::Pressed) => anim.press_scale,
            Some(Interaction::Hovered) => anim.hover_scale,
            _ => 1.0,
        };
        anim.interact += (target - anim.interact) * (1.0 - (-dt * 18.0).exp());
        anim.bounce = (anim.bounce - dt * 4.0).max(0.0);

        let mut scale = anim.interact;
        scale *= 1.0 + 0.25 * (anim.bounce * PI).sin();
        if let Some((amount, speed)) = anim.pulse {
            scale *= 1.0 + amount * (t * speed * TAU).sin();
        }
        if let Some(duration) = anim.pop_in {
            scale *= ease_out_back((t / duration.max(0.001)).min(1.0));
        }

        let mut degrees = anim.spin * t;
        if let Some((amount, speed)) = anim.wiggle {
            degrees += amount * (t * speed * TAU).sin();
        }

        let mut offset_y = 0.0;
        if let Some((amount, speed)) = anim.bob {
            offset_y = amount * (t * speed * TAU).sin();
        }

        let base = &anim.base;
        let new = UiTransform {
            translation: Val2::px(px_of(base.translation.x), px_of(base.translation.y) + offset_y),
            scale: base.scale * scale,
            rotation: base.rotation * Rot2::degrees(degrees),
        };
        if *transform != new {
            *transform = new;
        }
    }
}

/// Cycles the texture atlas index of an image (see [`crate::widgets::ImageView::animate_frames`]).
#[derive(Component, Clone, Debug)]
pub struct FrameAnimation {
    pub frames: Vec<usize>,
    pub fps: f32,
    pub looping: bool,
    elapsed: f32,
}

impl FrameAnimation {
    pub fn new(frames: Vec<usize>, fps: f32, looping: bool) -> Self {
        Self { frames, fps, looping, elapsed: 0.0 }
    }

    /// Plays the animation again from the first frame.
    pub fn restart(&mut self) {
        self.elapsed = 0.0;
    }
}

pub(crate) fn animate_frames(time: Res<Time>, mut query: Query<(&mut FrameAnimation, &mut ImageNode)>) {
    for (mut anim, mut image) in &mut query {
        if anim.frames.is_empty() {
            continue;
        }
        anim.elapsed += time.delta_secs();
        let mut frame = (anim.elapsed * anim.fps) as usize;
        frame = if anim.looping { frame % anim.frames.len() } else { frame.min(anim.frames.len() - 1) };
        let index = anim.frames[frame];
        if let Some(atlas) = &mut image.texture_atlas
            && atlas.index != index
        {
            atlas.index = index;
        }
    }
}
