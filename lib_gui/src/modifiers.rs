use bevy::{ecs::{observer::IntoEntityObserver, system::SystemId}, prelude::*};

use crate::{
    binding::{BoundBackground, BoundDisplay, BoundNode},
    interaction::{InteractiveColors, OwnedSystems},
    view::ParentLayout,
};

/// Pointer event an action reacts to (used internally by the `on_*` modifiers).
#[doc(hidden)]
#[derive(Clone, Copy)]
pub enum PointerTrigger {
    Click,
    Over,
    Out,
    Press,
    Release,
}

type Register = Box<dyn FnOnce(&mut Commands) -> SystemId>;
type EntityHook = Box<dyn FnOnce(&mut EntityCommands)>;

/// Style and behaviour shared by every widget.
///
/// You rarely use it directly: call the methods of [`Modifiable`] instead
/// (`.padding(..)`, `.background(..)`, `.on_tap(..)`, ...).
pub struct Modifiers {
    pub node: Node,
    background: Option<Color>,
    border_color: Option<Color>,
    shadow: Option<ShadowStyle>,
    gradient: Option<BackgroundGradient>,
    outline: Option<Outline>,
    transform: Option<UiTransform>,
    hovered: Option<Color>,
    pressed: Option<Color>,
    actions: Vec<(PointerTrigger, Register)>,
    hooks: Vec<EntityHook>,
}

impl Modifiers {
    pub fn new(node: Node) -> Self {
        Self {
            node,
            background: None,
            border_color: None,
            shadow: None,
            gradient: None,
            outline: None,
            transform: None,
            hovered: None,
            pressed: None,
            actions: Vec::new(),
            hooks: Vec::new(),
        }
    }

    /// Inserts everything onto `entity`. Called by widgets at the end of `build`.
    pub(crate) fn apply(mut self, parent: ParentLayout, entity: &mut EntityCommands) {
        if parent == ParentLayout::Overlay {
            self.node.grid_row = GridPlacement::start(1);
            self.node.grid_column = GridPlacement::start(1);
        }
        entity.insert(self.node);

        if let Some(color) = self.background {
            entity.insert(BackgroundColor(color));
        }
        if let Some(color) = self.border_color {
            entity.insert(BorderColor::all(color));
        }
        if let Some(shadow) = self.shadow {
            entity.insert(BoxShadow(vec![shadow]));
        }
        if let Some(gradient) = self.gradient {
            entity.insert(gradient);
        }
        if let Some(outline) = self.outline {
            entity.insert(outline);
        }
        if let Some(transform) = self.transform {
            entity.insert(transform);
        }
        if self.hovered.is_some() || self.pressed.is_some() {
            let normal = self.background.unwrap_or(Color::NONE);
            entity.insert((
                Interaction::default(),
                InteractiveColors {
                    normal,
                    hovered: self.hovered.unwrap_or(normal),
                    pressed: self.pressed.or(self.hovered).unwrap_or(normal),
                },
            ));
        }

        if !self.actions.is_empty() {
            let mut owned = Vec::with_capacity(self.actions.len());
            for (trigger, register) in self.actions {
                let id = register(&mut entity.commands());
                owned.push(id);
                let run = move |mut commands: Commands| commands.run_system(id);
                match trigger {
                    PointerTrigger::Click => entity.observe(move |_: On<Pointer<Click>>, c: Commands| run(c)),
                    PointerTrigger::Over => entity.observe(move |_: On<Pointer<Over>>, c: Commands| run(c)),
                    PointerTrigger::Out => entity.observe(move |_: On<Pointer<Out>>, c: Commands| run(c)),
                    PointerTrigger::Press => entity.observe(move |_: On<Pointer<Press>>, c: Commands| run(c)),
                    PointerTrigger::Release => entity.observe(move |_: On<Pointer<Release>>, c: Commands| run(c)),
                };
            }
            entity.insert(OwnedSystems(owned));
        }

        for hook in self.hooks {
            hook(entity);
        }
    }
}

/// SwiftUI-like modifiers, available on every widget.
///
/// ```ignore
/// text("Hello").padding(8.).background(Color::BLACK).corner_radius(6.)
/// ```
pub trait Modifiable: Sized {
    fn modifiers(&mut self) -> &mut Modifiers;

    /// Escape hatch: edit the underlying bevy [`Node`] directly.
    fn node(mut self, f: impl FnOnce(&mut Node)) -> Self {
        f(&mut self.modifiers().node);
        self
    }

    // ---------- spacing ----------

    fn padding(self, all: f32) -> Self {
        self.node(|n| n.padding = UiRect::all(px(all)))
    }
    /// Horizontal / vertical padding.
    fn padding_xy(self, horizontal: f32, vertical: f32) -> Self {
        self.node(|n| n.padding = UiRect::axes(px(horizontal), px(vertical)))
    }
    fn padding_rect(self, rect: UiRect) -> Self {
        self.node(|n| n.padding = rect)
    }
    fn margin(self, all: f32) -> Self {
        self.node(|n| n.margin = UiRect::all(px(all)))
    }
    fn margin_rect(self, rect: UiRect) -> Self {
        self.node(|n| n.margin = rect)
    }

    // ---------- size ----------

    /// Fixed size, like SwiftUI's `.frame(width:height:)`.
    fn frame(self, width: f32, height: f32) -> Self {
        self.node(|n| {
            n.width = px(width);
            n.height = px(height);
        })
    }
    fn width(self, width: impl Into<Val>) -> Self {
        let width = width.into();
        self.node(|n| n.width = width)
    }
    fn height(self, height: impl Into<Val>) -> Self {
        let height = height.into();
        self.node(|n| n.height = height)
    }
    fn min_width(self, width: f32) -> Self {
        self.node(|n| n.min_width = px(width))
    }
    fn min_height(self, height: f32) -> Self {
        self.node(|n| n.min_height = px(height))
    }
    fn max_width(self, width: f32) -> Self {
        self.node(|n| n.max_width = px(width))
    }
    fn max_height(self, height: f32) -> Self {
        self.node(|n| n.max_height = px(height))
    }
    /// Takes the whole available width.
    fn fill_width(self) -> Self {
        self.node(|n| n.width = percent(100))
    }
    /// Takes the whole available height.
    fn fill_height(self) -> Self {
        self.node(|n| n.height = percent(100))
    }
    /// Takes the whole available space (use it on your root view for a full screen UI).
    fn fill(self) -> Self {
        self.fill_width().fill_height()
    }
    /// Grows to take the remaining space in its stack.
    fn grow(self) -> Self {
        self.node(|n| n.flex_grow = 1.0)
    }
    fn aspect_ratio(self, ratio: f32) -> Self {
        self.node(|n| n.aspect_ratio = Some(ratio))
    }

    // ---------- positioning ----------

    /// Overrides the alignment chosen by the parent stack for this view.
    fn align_self(self, align: AlignSelf) -> Self {
        self.node(|n| n.align_self = align)
    }
    /// Horizontal alignment of this view inside a `zstack` or `grid` cell.
    fn justify_self(self, justify: JustifySelf) -> Self {
        self.node(|n| n.justify_self = justify)
    }
    /// Takes the view out of the layout and places it at `left, top` of its parent.
    fn absolute(self, left: f32, top: f32) -> Self {
        self.node(|n| {
            n.position_type = PositionType::Absolute;
            n.left = px(left);
            n.top = px(top);
        })
    }
    /// Moves the view without affecting the layout of its siblings.
    fn offset(self, x: f32, y: f32) -> Self {
        self.node(|n| {
            n.left = px(x);
            n.top = px(y);
        })
    }
    /// Clips children overflowing this view.
    fn clip(self) -> Self {
        self.node(|n| n.overflow = Overflow::clip())
    }
    /// Hides the view (and removes it from the layout).
    fn hidden(self, hidden: bool) -> Self {
        self.node(|n| {
            if hidden {
                n.display = Display::None;
            }
        })
    }

    // ---------- look ----------

    fn background(mut self, color: impl Into<Color>) -> Self {
        self.modifiers().background = Some(color.into());
        self
    }
    fn corner_radius(self, radius: f32) -> Self {
        self.node(|n| n.border_radius = BorderRadius::all(px(radius)))
    }
    /// Fully rounded ends (pill / circle).
    fn capsule(self) -> Self {
        self.node(|n| n.border_radius = BorderRadius::MAX)
    }
    fn border(mut self, width: f32, color: impl Into<Color>) -> Self {
        self.modifiers().border_color = Some(color.into());
        self.node(|n| n.border = UiRect::all(px(width)))
    }
    fn shadow(mut self, color: impl Into<Color>, blur: f32, x: f32, y: f32) -> Self {
        self.modifiers().shadow = Some(ShadowStyle {
            color: color.into(),
            x_offset: px(x),
            y_offset: px(y),
            spread_radius: px(0),
            blur_radius: px(blur),
        });
        self
    }
    /// Background used while the pointer is over the view.
    fn hover_background(mut self, color: impl Into<Color>) -> Self {
        self.modifiers().hovered = Some(color.into());
        self
    }
    /// Background used while the view is pressed.
    fn pressed_background(mut self, color: impl Into<Color>) -> Self {
        self.modifiers().pressed = Some(color.into());
        self
    }

    /// Individual corner radii (top-left, top-right, bottom-right, bottom-left).
    fn corner_radii(self, top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        self.node(|n| n.border_radius = BorderRadius::px(top_left, top_right, bottom_right, bottom_left))
    }
    /// Vertical gradient from `top` to `bottom`.
    fn gradient(self, top: impl Into<Color>, bottom: impl Into<Color>) -> Self {
        self.linear_gradient(180.0, vec![top.into(), bottom.into()])
    }
    /// Linear gradient following CSS angles: 0 = towards the top, 90 = towards the right, 180 = towards the bottom.
    fn linear_gradient(mut self, angle_deg: f32, colors: Vec<Color>) -> Self {
        let stops = colors.into_iter().map(ColorStop::auto).collect();
        self.modifiers().gradient = Some(LinearGradient::new(angle_deg.to_radians(), stops).into());
        self
    }
    /// Outline drawn outside the border, without affecting layout.
    fn outline(mut self, width: f32, color: impl Into<Color>) -> Self {
        self.modifiers().outline = Some(Outline::new(px(width), px(0), color.into()));
        self
    }
    /// Draw order among siblings (higher = in front).
    fn z_index(self, z: i32) -> Self {
        self.insert(ZIndex(z))
    }
    /// Draw order relative to the whole UI (for popups, tooltips...).
    fn global_z_index(self, z: i32) -> Self {
        self.insert(GlobalZIndex(z))
    }
    /// Visual scale (does not affect layout).
    fn scale(mut self, scale: f32) -> Self {
        self.modifiers().transform.get_or_insert(UiTransform::IDENTITY).scale = Vec2::splat(scale);
        self
    }
    /// Visual rotation in degrees (does not affect layout).
    fn rotation(mut self, degrees: f32) -> Self {
        self.modifiers().transform.get_or_insert(UiTransform::IDENTITY).rotation = Rot2::degrees(degrees);
        self
    }
    /// Visual translation in pixels (does not affect layout).
    fn translate(mut self, x: f32, y: f32) -> Self {
        self.modifiers().transform.get_or_insert(UiTransform::IDENTITY).translation = Val2::px(x, y);
        self
    }
    /// Invisible but still takes its space in the layout (unlike `hidden`).
    fn invisible(self) -> Self {
        self.insert(Visibility::Hidden)
    }
    /// Per-side padding.
    fn padding_each(self, top: f32, right: f32, bottom: f32, left: f32) -> Self {
        self.node(|n| n.padding = UiRect::new(px(left), px(right), px(top), px(bottom)))
    }
    fn min_size(self, width: f32, height: f32) -> Self {
        self.min_width(width).min_height(height)
    }
    fn max_size(self, width: f32, height: f32) -> Self {
        self.max_width(width).max_height(height)
    }
    /// Won't shrink when space is lacking.
    fn no_shrink(self) -> Self {
        self.node(|n| n.flex_shrink = 0.0)
    }

    // ---------- events ----------
    //
    // Actions are ordinary Bevy systems: they can take `Res`, `ResMut`, `Commands`, `Query`...
    //     .on_tap(|mut score: ResMut<Score>| score.0 += 1)

    fn on_tap<M>(self, action: impl IntoSystem<(), (), M> + 'static) -> Self {
        self.on_pointer(PointerTrigger::Click, action)
    }
    fn on_hover<M>(self, action: impl IntoSystem<(), (), M> + 'static) -> Self {
        self.on_pointer(PointerTrigger::Over, action)
    }
    fn on_hover_end<M>(self, action: impl IntoSystem<(), (), M> + 'static) -> Self {
        self.on_pointer(PointerTrigger::Out, action)
    }
    fn on_press<M>(self, action: impl IntoSystem<(), (), M> + 'static) -> Self {
        self.on_pointer(PointerTrigger::Press, action)
    }
    fn on_release<M>(self, action: impl IntoSystem<(), (), M> + 'static) -> Self {
        self.on_pointer(PointerTrigger::Release, action)
    }

    #[doc(hidden)]
    fn on_pointer<M>(mut self, trigger: PointerTrigger, action: impl IntoSystem<(), (), M> + 'static) -> Self {
        self.modifiers()
            .actions
            .push((trigger, Box::new(move |commands: &mut Commands| commands.register_system(action))));
        self
    }

    /// Adds a raw Bevy observer (for events not covered by the `on_*` modifiers).
    fn observe<M>(self, observer: impl IntoEntityObserver<M>) -> Self {
        self.with_entity(move |e| {
            e.observe(observer);
        })
    }

    // ---------- ecs ----------

    /// Inserts extra components (markers, `DespawnOnExit(..)`, ...).
    fn insert(self, bundle: impl Bundle) -> Self {
        self.with_entity(move |e| {
            e.insert(bundle);
        })
    }
    fn name(self, name: impl Into<String>) -> Self {
        self.insert(Name::new(name.into()))
    }
    /// Full access to the spawned entity.
    fn with_entity(mut self, f: impl FnOnce(&mut EntityCommands) + 'static) -> Self {
        self.modifiers().hooks.push(Box::new(f));
        self
    }

    // ---------- reactive ----------

    /// Shows the view only while `condition` returns `true` for resource `R`.
    ///
    /// ```ignore
    /// text("Game Over").show_if(|state: &GameInfo| state.lives == 0)
    /// ```
    fn show_if<R: Resource>(self, condition: impl Fn(&R) -> bool + Send + Sync + 'static) -> Self {
        self.show_if_world(move |world| world.get_resource::<R>().is_some_and(&condition))
    }
    /// Background computed from resource `R`, updated automatically.
    fn bind_background<R: Resource>(self, f: impl Fn(&R) -> Color + Send + Sync + 'static) -> Self {
        self.insert(BoundBackground(Box::new(move |world| world.get_resource::<R>().map(&f))))
    }
    /// Edits the node from resource `R` every frame (width of a bar, alignment...).
    ///
    /// ```ignore
    /// rectangle(Color::RED).height(8.).bind_node(|hp: &Health, node| node.width = percent(hp.0))
    /// ```
    fn bind_node<R: Resource>(self, f: impl Fn(&R, &mut Node) + Send + Sync + 'static) -> Self {
        self.insert(BoundNode(Box::new(move |world, node| {
            if let Some(res) = world.get_resource::<R>() {
                f(res, node);
            }
        })))
    }

    /// Like [`Modifiable::show_if`] but with read access to the whole `World`.
    fn show_if_world(self, condition: impl Fn(&World) -> bool + Send + Sync + 'static) -> Self {
        self.with_entity(move |e| {
            e.insert(BoundDisplay::new(condition));
        })
    }
}
