//! `cargo run --example demo`

use bevy::prelude::*;
use lib_gui::prelude::*;

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Resource)]
struct Settings {
    name: String,
    music: bool,
    vibrations: bool,
    volume: f32,
    difficulty: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self { name: String::new(), music: true, vibrations: false, volume: 70.0, difficulty: 1 }
    }
}

const GRAY: Color = Color::srgb(0.6, 0.6, 0.65);
const RED: Color = Color::srgb(0.9, 0.25, 0.3);
const GREEN: Color = Color::srgb(0.2, 0.78, 0.35);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, GuiPlugin))
        .init_resource::<Score>()
        .init_resource::<Settings>()
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Bevy's built-in font has no accents (é, à...). Put a .ttf in `assets/` to display them:
    // commands.insert_resource(GuiFont(asset_server.load("fonts/MyFont.ttf")));

    commands.spawn_view(
        hstack((menu(), settings(), inventory()))
            .spacing(20.0)
            .align(Align::Start)
            .fill()
            .center()
            .gradient(Color::srgb(0.08, 0.09, 0.14), Color::srgb(0.02, 0.02, 0.04)),
    );
}

/// Buttons, dynamic text, progress bar, conditional view.
fn menu() -> impl View {
    card((
        text("SigmaSix").large_title().text_shadow(Color::BLACK, 2.0, 2.0),
        text("Menu principal").caption().color(GRAY),
        divider(),
        dyn_text(|s: &Score| format!("Score : {}", s.0))
            .headline()
            .bind_color(|s: &Score| if s.0 >= 20 { GREEN } else { Color::WHITE }),
        hstack((
            button("-1", |mut s: ResMut<Score>| s.0 = s.0.saturating_sub(1)).tint(RED),
            button("+1", |mut s: ResMut<Score>| s.0 += 1),
            button("Reset", |mut s: ResMut<Score>| s.0 = 0).plain(),
        )),
        progress_bar(|s: &Score| s.0 as f32 / 20.0)
            .fill_width()
            .bind_tint(|s: &Score| if s.0 >= 20 { GREEN } else { Color::srgb(0.0, 0.48, 1.0) }),
        text("Objectif atteint !").color(GREEN).bold().show_if(|s: &Score| s.0 >= 20),
        spacer(),
        button("Quitter", |mut exit: MessageWriter<AppExit>| {
            exit.write(AppExit::Success);
        })
            .tint(Color::srgb(0.25, 0.25, 0.3))
            .fill_width(),
    ))
        .spacing(12.0)
        .width(px(280))
        .min_height(420.0)
}

/// Controls bound to the `Settings` resource.
fn settings() -> impl View {
    card((
        text("Reglages").title(),
        text_field("Pseudo", |s: &Settings| s.name.clone(), |s, v| s.name = v).max_length(16).fill_width(),
        dyn_text(|s: &Settings| {
            if s.name.is_empty() { "Entre ton pseudo".into() } else { format!("Salut {} !", s.name) }
        })
        .caption()
        .color(GRAY),
        divider(),
        toggle("Musique", |s: &Settings| s.music, |s, on| s.music = on).fill_width(),
        checkbox("Vibrations", |s: &Settings| s.vibrations, |s, on| s.vibrations = on),
        hstack((
            text("Volume"),
            slider(0.0..=100.0, |s: &Settings| s.volume, |s, v| s.volume = v).step(1.0).grow(),
            dyn_text(|s: &Settings| format!("{:.0}%", s.volume)).width(px(44)),
        ))
        .spacing(12.0)
        .fill_width(),
        text("Difficulte").caption().color(GRAY),
        picker(["Facile", "Normal", "Difficile"], |s: &Settings| s.difficulty, |s, i| s.difficulty = i)
            .fill_width(),
    ))
    .spacing(14.0)
    .width(px(340))
}

/// Grid, zstack, badge, scroll view, gradient, transforms.
fn inventory() -> impl View {
    let colors = [RED, GREEN, Color::srgb(0.95, 0.7, 0.2), Color::srgb(0.4, 0.5, 1.0)];

    card((
        text("Inventaire").title(),
        grid(
            4,
            for_each(0..8, |i| {
                zstack((
                    rectangle(colors[i % 4].darker(0.2)).frame(56.0, 56.0).corner_radius(10.0),
                    text(format!("{}", i + 1)).headline(),
                    (i % 3 == 0).then(|| badge("x3").align_self(AlignSelf::Start).justify_self(JustifySelf::End).translate(6.0, -6.0)),
                ))
                .hover_background(Color::srgba(1.0, 1.0, 1.0, 0.1))
                .corner_radius(10.0)
                .on_tap(move |mut s: ResMut<Score>| s.0 += i as u32)
            }),
        )
        .spacing(8.0),
        text("Journal").headline(),
        scroll_view(for_each(1..=30, |i| {
            hstack((circle(8.0, colors[i % 4]), text(format!("Evenement #{i}")).body())).spacing(8.0).padding(4.0)
        }))
        .spacing(0.0)
        .height(px(140))
        .fill_width()
        .padding(6.0)
        .background(Color::srgb(0.08, 0.08, 0.1))
        .corner_radius(8.0),
        zstack(text("ZStack + degrade").bold().rotation(-4.0))
            .frame(260.0, 56.0)
            .linear_gradient(90.0, vec![Color::srgb(0.5, 0.2, 0.9), Color::srgb(0.1, 0.6, 1.0)])
            .corner_radius(12.0)
            .outline(2.0, Color::WHITE),
    ))
    .spacing(12.0)
    .width(px(300))
}
