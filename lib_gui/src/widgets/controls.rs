//! Controls bound to a resource: each one takes a getter (`|r: &R| -> value`)
//! and a setter (`|r: &mut R, value|`), like a SwiftUI `Binding`.

use std::{ops::RangeInclusive, sync::Arc};

use bevy::prelude::*;

use super::{Align, Stack, circle, hstack, rectangle, spacer, text, zstack};
use crate::{
    MutResource,
    modifiers::{Modifiable, Modifiers},
    view::{BuildCx, ParentLayout, View},
};

pub(crate) const ACCENT: Color = Color::srgb(0.0, 0.48, 1.0);
const OFF: Color = Color::srgb(0.3, 0.3, 0.34);
const TRACK: Color = Color::srgb(0.22, 0.22, 0.26);

type Getter<R, T> = Arc<dyn Fn(&R) -> T + Send + Sync>;
type Setter<R, T> = Arc<dyn Fn(&mut R, T) + Send + Sync>;

// ============================================================================
// Toggle
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum ToggleStyle {
    Switch,
    Checkbox,
}

/// An on/off control. Created with [`toggle`] (switch) or [`checkbox`].
pub struct ToggleView<R: MutResource> {
    label: Option<String>,
    get: Getter<R, bool>,
    set: Setter<R, bool>,
    style: ToggleStyle,
    tint: Color,
    off_color: Color,
    mods: Modifiers,
}

impl<R: MutResource> Modifiable for ToggleView<R> {
    fn modifiers(&mut self) -> &mut Modifiers {
        &mut self.mods
    }
}

/// An iOS-like switch with a label (SwiftUI's `Toggle`). Pass `""` for no label.
///
/// ```ignore
/// toggle("Music", |s: &Settings| s.music, |s, on| s.music = on)
/// ```
pub fn toggle<R: MutResource>(
    label: impl Into<String>,
    get: impl Fn(&R) -> bool + Send + Sync + 'static,
    set: impl Fn(&mut R, bool) + Send + Sync + 'static,
) -> ToggleView<R> {
    let label: String = label.into();
    ToggleView {
        label: (!label.is_empty()).then_some(label),
        get: Arc::new(get),
        set: Arc::new(set),
        style: ToggleStyle::Switch,
        tint: Color::srgb(0.2, 0.78, 0.35),
        off_color: OFF,
        mods: Modifiers::new(Node { align_items: AlignItems::Center, column_gap: px(10), ..default() }),
    }
}

/// A checkbox with a label.
pub fn checkbox<R: MutResource>(
    label: impl Into<String>,
    get: impl Fn(&R) -> bool + Send + Sync + 'static,
    set: impl Fn(&mut R, bool) + Send + Sync + 'static,
) -> ToggleView<R> {
    let mut view = toggle(label, get, set);
    view.style = ToggleStyle::Checkbox;
    view.tint = ACCENT;
    view
}

impl<R: MutResource> ToggleView<R> {
    /// Color when on.
    pub fn tint(mut self, color: impl Into<Color>) -> Self {
        self.tint = color.into();
        self
    }
    /// Color of the switch track when off.
    pub fn off_color(mut self, color: impl Into<Color>) -> Self {
        self.off_color = color.into();
        self
    }
    /// Displays it as a checkbox instead of a switch.
    pub fn checkbox_style(mut self) -> Self {
        self.style = ToggleStyle::Checkbox;
        self
    }
    /// Displays it as a switch.
    pub fn switch_style(mut self) -> Self {
        self.style = ToggleStyle::Switch;
        self
    }
}

impl<R: MutResource> View for ToggleView<R> {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let Self { label, get, set, style, tint, off_color, mods } = self;
        let (get_bg, get_node, get_check) = (get.clone(), get.clone(), get.clone());
        let label = label.map(text);

        let content = match style {
            ToggleStyle::Switch => {
                let switch = hstack(circle(20.0, Color::WHITE).shadow(Color::srgba(0.0, 0.0, 0.0, 0.3), 3.0, 0.0, 1.0))
                    .spacing(0.0)
                    .frame(44.0, 24.0)
                    .padding(2.0)
                    .capsule()
                    .no_shrink()
                    .bind_background(move |r: &R| if get_bg(r) { tint } else { off_color })
                    .bind_node(move |r: &R, n| {
                        n.justify_content = if get_node(r) { JustifyContent::End } else { JustifyContent::Start };
                    });
                // Label on the left, switch pushed to the right.
                let push = label.is_some().then(spacer);
                (label, push, switch).boxed()
            }
            ToggleStyle::Checkbox => {
                let check = rectangle(Color::WHITE)
                    .frame(10.0, 10.0)
                    .corner_radius(2.0)
                    .show_if(move |r: &R| get_check(r));
                let square = zstack(check)
                    .frame(22.0, 22.0)
                    .corner_radius(5.0)
                    .border(2.0, tint)
                    .no_shrink()
                    .bind_background(move |r: &R| if get_bg(r) { tint } else { Color::NONE });
                (square, label).boxed()
            }
        };

        Stack { layout: ParentLayout::Row, content, mods }
            .on_tap(move |mut res: ResMut<R>| {
                let value = get(&res);
                set(&mut res, !value);
            })
            .build(cx, out);
    }
}

// ============================================================================
// Slider
// ============================================================================

/// A draggable slider. Created with [`slider`].
pub struct SliderView<R: MutResource> {
    range: RangeInclusive<f32>,
    step: Option<f32>,
    get: Getter<R, f32>,
    set: Setter<R, f32>,
    tint: Color,
    track: Color,
    mods: Modifiers,
}

impl<R: MutResource> Modifiable for SliderView<R> {
    fn modifiers(&mut self) -> &mut Modifiers {
        &mut self.mods
    }
}

/// A slider editing an `f32` of resource `R` within `range` (SwiftUI's `Slider`).
///
/// ```ignore
/// slider(0.0..=1.0, |s: &Settings| s.volume, |s, v| s.volume = v)
/// ```
pub fn slider<R: MutResource>(
    range: RangeInclusive<f32>,
    get: impl Fn(&R) -> f32 + Send + Sync + 'static,
    set: impl Fn(&mut R, f32) + Send + Sync + 'static,
) -> SliderView<R> {
    SliderView {
        range,
        step: None,
        get: Arc::new(get),
        set: Arc::new(set),
        tint: ACCENT,
        track: TRACK,
        mods: Modifiers::new(Node {
            width: px(200),
            height: px(20),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        }),
    }
}

impl<R: MutResource> SliderView<R> {
    /// Snaps the value to multiples of `step`.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
    /// Color of the filled part.
    pub fn tint(mut self, color: impl Into<Color>) -> Self {
        self.tint = color.into();
        self
    }
    /// Color of the empty part.
    pub fn track_color(mut self, color: impl Into<Color>) -> Self {
        self.track = color.into();
        self
    }
}

fn fraction(value: f32, range: &RangeInclusive<f32>) -> f32 {
    let span = range.end() - range.start();
    if span <= 0.0 { 0.0 } else { ((value - range.start()) / span).clamp(0.0, 1.0) }
}

/// Horizontal position of the pointer inside `entity`, from 0 (left) to 1 (right).
fn pointer_fraction(
    entity: Entity,
    position: Vec2,
    nodes: &Query<(&ComputedNode, &UiGlobalTransform)>,
) -> Option<f32> {
    let (node, transform) = nodes.get(entity).ok()?;
    let physical = position / node.inverse_scale_factor();
    node.normalize_point(*transform, physical).map(|p| (p.x + 0.5).clamp(0.0, 1.0))
}

impl<R: MutResource> View for SliderView<R> {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let Self { range, step, get, set, tint, track, mods } = self;

        let (get_fill, get_thumb) = (get.clone(), get);
        let (range_fill, range_thumb) = (range.clone(), range.clone());

        let fill = rectangle(tint)
            .fill_height()
            .capsule()
            .bind_node(move |r: &R, n| n.width = percent(fraction(get_fill(r), &range_fill) * 100.0));
        let track = hstack(fill).spacing(0.0).align(Align::Stretch).fill_width().height(px(6)).capsule().background(track);
        let thumb = circle(18.0, Color::WHITE)
            .shadow(Color::srgba(0.0, 0.0, 0.0, 0.35), 4.0, 0.0, 1.0)
            .node(|n| {
                n.position_type = PositionType::Absolute;
                n.top = px(1);
                n.margin.left = px(-9);
            })
            .bind_node(move |r: &R, n| n.left = percent(fraction(get_thumb(r), &range_thumb) * 100.0));

        let apply = move |res: &mut R, t: f32| {
            let mut value = range.start() + t * (range.end() - range.start());
            if let Some(step) = step.filter(|s| *s > 0.0) {
                value = range.start() + ((value - range.start()) / step).round() * step;
            }
            set(res, value.clamp(*range.start(), *range.end()));
        };
        let apply_drag = apply.clone();

        Stack { layout: ParentLayout::Row, content: (track, thumb), mods }
            .observe(move |press: On<Pointer<Press>>, nodes: Query<(&ComputedNode, &UiGlobalTransform)>, mut res: ResMut<R>| {
                if let Some(t) = pointer_fraction(press.entity, press.pointer_location.position, &nodes) {
                    apply(&mut res, t);
                }
            })
            .observe(move |drag: On<Pointer<Drag>>, nodes: Query<(&ComputedNode, &UiGlobalTransform)>, mut res: ResMut<R>| {
                if let Some(t) = pointer_fraction(drag.entity, drag.pointer_location.position, &nodes) {
                    apply_drag(&mut res, t);
                }
            })
            .build(cx, out);
    }
}

// ============================================================================
// Progress bar
// ============================================================================

/// A read-only progress bar. Created with [`progress_bar`].
pub struct ProgressView<R: MutResource> {
    get: Getter<R, f32>,
    tint: Color,
    mods: Modifiers,
}

impl<R: MutResource> Modifiable for ProgressView<R> {
    fn modifiers(&mut self) -> &mut Modifiers {
        &mut self.mods
    }
}

/// A progress bar showing a value between 0 and 1 (SwiftUI's `ProgressView(value:)`).
///
/// ```ignore
/// progress_bar(|p: &Player| p.health / p.max_health).tint(Color::srgb(0.9, 0.2, 0.2))
/// ```
pub fn progress_bar<R: MutResource>(get: impl Fn(&R) -> f32 + Send + Sync + 'static) -> ProgressView<R> {
    ProgressView {
        get: Arc::new(get),
        tint: ACCENT,
        mods: Modifiers::new(Node {
            width: px(200),
            height: px(8),
            align_items: AlignItems::Stretch,
            flex_shrink: 0.0,
            border_radius: BorderRadius::MAX,
            overflow: Overflow::clip(),
            ..default()
        }),
    }
    .background(TRACK)
}

impl<R: MutResource> ProgressView<R> {
    /// Color of the filled part (the background modifier sets the empty part).
    pub fn tint(mut self, color: impl Into<Color>) -> Self {
        self.tint = color.into();
        self
    }
    /// Fill color computed from the resource (e.g. green → red as health drops).
    pub fn bind_tint(self, f: impl Fn(&R) -> Color + Send + Sync + 'static) -> BoundTintProgress<R> {
        BoundTintProgress { bar: self, tint: Arc::new(f) }
    }
}

/// A [`ProgressView`] whose fill color follows the resource, see [`ProgressView::bind_tint`].
pub struct BoundTintProgress<R: MutResource> {
    bar: ProgressView<R>,
    tint: Getter<R, Color>,
}

impl<R: MutResource> Modifiable for BoundTintProgress<R> {
    fn modifiers(&mut self) -> &mut Modifiers {
        &mut self.bar.mods
    }
}

impl<R: MutResource> View for BoundTintProgress<R> {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let tint = self.tint;
        self.bar.build_with(cx, out, Some(tint));
    }
}

impl<R: MutResource> ProgressView<R> {
    fn build_with(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>, tint: Option<Getter<R, Color>>) {
        let get = self.get;
        let mut fill = rectangle(self.tint)
            .capsule()
            .bind_node(move |r: &R, n| n.width = percent(get(r).clamp(0.0, 1.0) * 100.0));
        if let Some(tint) = tint {
            fill = fill.bind_background(move |r: &R| tint(r));
        }
        Stack { layout: ParentLayout::Row, content: fill, mods: self.mods }.build(cx, out);
    }
}

impl<R: MutResource> View for ProgressView<R> {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        self.build_with(cx, out, None);
    }
}

// ============================================================================
// Segmented picker
// ============================================================================

/// One choice among several. Created with [`picker`].
pub struct PickerView<R: MutResource> {
    options: Vec<String>,
    get: Getter<R, usize>,
    set: Setter<R, usize>,
    tint: Color,
    mods: Modifiers,
}

impl<R: MutResource> Modifiable for PickerView<R> {
    fn modifiers(&mut self) -> &mut Modifiers {
        &mut self.mods
    }
}

/// A segmented control selecting an index (SwiftUI's `Picker` with `.segmented` style).
///
/// ```ignore
/// picker(["Easy", "Normal", "Hard"], |s: &Settings| s.difficulty, |s, i| s.difficulty = i)
/// ```
pub fn picker<R: MutResource>(
    options: impl IntoIterator<Item = impl Into<String>>,
    get: impl Fn(&R) -> usize + Send + Sync + 'static,
    set: impl Fn(&mut R, usize) + Send + Sync + 'static,
) -> PickerView<R> {
    PickerView {
        options: options.into_iter().map(Into::into).collect(),
        get: Arc::new(get),
        set: Arc::new(set),
        tint: ACCENT,
        mods: Modifiers::new(Node {
            padding: UiRect::all(px(3)),
            column_gap: px(2),
            border_radius: BorderRadius::all(px(8)),
            flex_shrink: 0.0,
            ..default()
        }),
    }
    .background(TRACK)
}

impl<R: MutResource> PickerView<R> {
    /// Color of the selected segment.
    pub fn tint(mut self, color: impl Into<Color>) -> Self {
        self.tint = color.into();
        self
    }
}

impl<R: MutResource> View for PickerView<R> {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        let tint = self.tint;
        let segments: Vec<_> = self
            .options
            .into_iter()
            .enumerate()
            .map(|(index, option)| {
                let (get, set) = (self.get.clone(), self.set.clone());
                hstack(text(option).semibold())
                    .padding_xy(14.0, 6.0)
                    .corner_radius(6.0)
                    .grow()
                    .center()
                    .bind_background(move |r: &R| if get(r) == index { tint } else { Color::NONE })
                    .on_tap(move |mut res: ResMut<R>| set(&mut res, index))
            })
            .collect();

        Stack { layout: ParentLayout::Row, content: segments, mods: self.mods }.build(cx, out);
    }
}
