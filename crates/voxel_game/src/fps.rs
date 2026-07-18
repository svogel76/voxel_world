//! On-screen FPS readout from Bevy frame-time diagnostics.

use bevy::{
    color::palettes::css::GOLD,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    text::FontSize,
};

pub struct FpsPlugin;

impl Plugin for FpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, spawn_fps_text)
            .add_systems(Update, update_fps_text);
    }
}

#[derive(Component)]
struct FpsText;

fn spawn_fps_text(mut commands: Commands) {
    commands
        .spawn((
            Text::new("FPS: "),
            TextFont {
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextColor(Color::srgb(0.85, 0.85, 0.85)),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                ..default()
            },
        ))
        .with_child((
            TextSpan::default(),
            TextFont {
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextColor(GOLD.into()),
            FpsText,
        ));
}

fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut TextSpan, With<FpsText>>,
) {
    for mut span in &mut query {
        if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS)
            && let Some(value) = fps.smoothed()
        {
            **span = format!("{value:>4.0}");
        }
    }
}
