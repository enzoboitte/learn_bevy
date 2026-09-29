use bevy::prelude::*;

/// Layout of the parent a view is being built into.
///
/// Some views adapt to it (a `divider()` is horizontal in a `vstack`,
/// vertical in an `hstack`; children of a `zstack` are stacked on top of each other).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ParentLayout {
    #[default]
    Root,
    Column,
    Row,
    Overlay,
}

/// Context passed down while a view tree is turned into entities.
pub struct BuildCx<'a, 'w, 's> {
    pub commands: &'a mut Commands<'w, 's>,
    pub parent: ParentLayout,
}

/// Anything that can be turned into UI entities.
///
/// Implemented by every widget (`text`, `button`, `vstack`, ...) and by
/// tuples, `Vec`, `Option` and `()` so they can be used as a list of children.
pub trait View {
    /// Spawns the view and pushes its root entities (in order) into `out`.
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>);

    /// Erases the concrete type, handy for `if / else` branches returning different views.
    fn boxed(self) -> AnyView
    where
        Self: Sized + 'static,
    {
        AnyView(Box::new(self))
    }
}

trait DynView {
    fn build_boxed(self: Box<Self>, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>);
}

impl<V: View> DynView for V {
    fn build_boxed(self: Box<Self>, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        (*self).build(cx, out);
    }
}

/// A type-erased view, see [`View::boxed`].
pub struct AnyView(Box<dyn DynView>);

impl View for AnyView {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        self.0.build_boxed(cx, out);
    }
}

impl View for () {
    fn build(self, _: &mut BuildCx<'_, '_, '_>, _: &mut Vec<Entity>) {}
}

impl<V: View> View for Option<V> {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        if let Some(view) = self {
            view.build(cx, out);
        }
    }
}

impl<V: View> View for Vec<V> {
    fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
        for view in self {
            view.build(cx, out);
        }
    }
}

macro_rules! impl_view_for_tuple {
    ($($name:ident),+) => {
        impl<$($name: View),+> View for ($($name,)+) {
            #[allow(non_snake_case)]
            fn build(self, cx: &mut BuildCx<'_, '_, '_>, out: &mut Vec<Entity>) {
                let ($($name,)+) = self;
                $($name.build(cx, out);)+
            }
        }
    };
}

impl_view_for_tuple!(A);
impl_view_for_tuple!(A, B);
impl_view_for_tuple!(A, B, C);
impl_view_for_tuple!(A, B, C, D);
impl_view_for_tuple!(A, B, C, D, E);
impl_view_for_tuple!(A, B, C, D, E, F);
impl_view_for_tuple!(A, B, C, D, E, F, G);
impl_view_for_tuple!(A, B, C, D, E, F, G, H);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

/// SwiftUI's `ForEach`: builds one view per item.
pub fn for_each<T, V: View>(items: impl IntoIterator<Item = T>, f: impl FnMut(T) -> V) -> Vec<V> {
    items.into_iter().map(f).collect()
}

/// Spawns views from `Commands`.
pub trait SpawnViewExt {
    /// Spawns `view` as a UI root and returns its entity.
    ///
    /// If the view produces several roots they are wrapped in a full-screen container.
    fn spawn_view(&mut self, view: impl View) -> Entity;

    /// Spawns `view` as children of `parent`.
    fn spawn_view_in(&mut self, parent: Entity, view: impl View);
}

impl SpawnViewExt for Commands<'_, '_> {
    fn spawn_view(&mut self, view: impl View) -> Entity {
        let mut out = Vec::new();
        view.build(&mut BuildCx { commands: self, parent: ParentLayout::Root }, &mut out);

        match out.as_slice() {
            [single] => *single,
            _ => self
                .spawn(Node { width: percent(100), height: percent(100), ..default() })
                .add_children(&out)
                .id(),
        }
    }

    fn spawn_view_in(&mut self, parent: Entity, view: impl View) {
        let mut out = Vec::new();
        view.build(&mut BuildCx { commands: self, parent: ParentLayout::Root }, &mut out);
        self.entity(parent).add_children(&out);
    }
}
