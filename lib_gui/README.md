# lib_gui

Interfaces Bevy (0.19) déclaratives, à la SwiftUI.

```rust
use bevy::prelude::*;
use lib_gui::prelude::*;

#[derive(Resource, Default)]
struct Score(u32);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, GuiPlugin))
        .init_resource::<Score>()
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_view(
        vstack((
            text("Hello Bevy").title(),
            dyn_text(|s: &Score| format!("Score : {}", s.0)),
            button("+1", |mut s: ResMut<Score>| s.0 += 1),
        ))
        .fill()
        .center(),
    );
}
```

Exemple complet : `cargo run --example demo`

## Widgets

| Widget | Équivalent SwiftUI |
|---|---|
| `vstack`, `hstack`, `zstack` | `VStack`, `HStack`, `ZStack` |
| `scroll_view`, `hscroll_view` | `ScrollView` |
| `grid(colonnes, ..)` | `LazyVGrid` |
| `card(..)` | panneau arrondi avec ombre |
| `text("..")`, `dyn_text(\|r: &R\| ..)` | `Text` |
| `button("..", système)`, `button_view(vue, système)` | `Button` |
| `toggle`, `checkbox` | `Toggle` |
| `slider(range, get, set)` | `Slider` |
| `progress_bar(\|r: &R\| 0.0..1.0)` | `ProgressView` |
| `picker([..], get, set)` | `Picker` segmenté |
| `text_field("..", get, set)` | `TextField` |
| `image(handle)`, `rectangle(color)`, `circle(d, color)` | `Image`, `Rectangle`, `Circle` |
| `spacer()`, `divider()`, `badge("..")` | `Spacer`, `Divider`, `.badge` |
| `for_each(iter, \|x\| vue)` | `ForEach` |

Les enfants sont des tuples (jusqu'à 16), des `Vec`, des `Option` (`cond.then(|| vue)`)
ou des `AnyView` (`vue.boxed()` pour des branches `if/else`).

## Modificateurs (tous les widgets)

- **Taille** : `frame`, `width`, `height`, `min_*`, `max_*`, `fill`, `fill_width`, `fill_height`, `grow`, `aspect_ratio`, `no_shrink`
- **Espacement** : `padding`, `padding_xy`, `padding_each`, `margin`
- **Position** : `align_self`, `justify_self`, `absolute`, `offset`, `z_index`, `global_z_index`, `clip`
- **Apparence** : `background`, `gradient`, `linear_gradient`, `corner_radius`, `corner_radii`, `capsule`, `border`, `outline`, `shadow`, `hover_background`, `pressed_background`
- **Transformations** : `scale`, `rotation`, `translate`
- **Visibilité** : `hidden`, `invisible`, `show_if::<R>(..)`
- **Événements** (ce sont des systèmes Bevy) : `on_tap`, `on_hover`, `on_hover_end`, `on_press`, `on_release`, `observe`
- **Réactif** : `bind_background::<R>`, `bind_node::<R>`, `show_if::<R>`
- **ECS** : `insert(composant)`, `name`, `with_entity`, `node(|n| ..)`

Les widgets ont aussi leurs propres modificateurs : `text(..).title().bold().italic().color(..).bind_color(..)`,
`button(..).tint(..).plain()`, `slider(..).step(..)`, `text_field(..).password().max_length(..)`,
`vstack(..).spacing(..).align(..).justify(..).wrap()`, etc.

## Police

La police intégrée de Bevy n'a pas d'accents. Pour les afficher :

```rust
commands.insert_resource(GuiFont(asset_server.load("fonts/MaPolice.ttf")));
```
