//! Phase-2 mood lighting ported from `world_generator` reference_scene.

use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    camera::Hdr,
    core_pipeline::tonemapping::Tonemapping,
    light::{CascadeShadowConfigBuilder, FogVolume, VolumetricFog, VolumetricLight},
    pbr::ScreenSpaceAmbientOcclusion,
    post_process::bloom::Bloom,
    prelude::*,
};

/// Dark forest clear color + cool, low ambient (reference_scene mood).
pub fn insert_mood_resources(app: &mut App) {
    app.insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.08)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.22, 0.26, 0.32),
            brightness: 55.0,
            ..default()
        });
}

/// Key + fill directional lights, CSM, volumetric light, and a fog volume near origin.
pub fn setup_lights(mut commands: Commands) {
    let look_target = Vec3::new(0.0, 4.0, 0.0);

    // Participating medium for volumetric god rays around the play / vegetation area.
    commands.spawn((
        Name::new("FogVolume"),
        FogVolume {
            fog_color: Color::srgb(0.92, 0.94, 0.97),
            density_factor: 0.08,
            absorption: 0.12,
            scattering: 0.7,
            ..default()
        },
        Transform::from_xyz(0.0, 12.0, 0.0).with_scale(Vec3::new(48.0, 40.0, 48.0)),
    ));

    commands.spawn((
        Name::new("KeySun"),
        DirectionalLight {
            illuminance: 80_000.0,
            shadow_maps_enabled: true,
            color: Color::srgb(1.0, 0.96, 0.88),
            ..default()
        },
        Transform::from_xyz(12.0, 40.0, 28.0).looking_at(look_target, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.5,
            maximum_distance: 90.0,
            first_cascade_far_bound: 18.0,
            ..default()
        }
        .build(),
        VolumetricLight,
    ));

    commands.spawn((
        Name::new("CoolFill"),
        DirectionalLight {
            illuminance: 5_500.0,
            shadow_maps_enabled: false,
            color: Color::srgb(0.40, 0.48, 0.60),
            ..default()
        },
        Transform::from_xyz(-18.0, 22.0, -10.0).looking_at(look_target, Vec3::Y),
    ));
}

/// Camera postprocessing + SSAO matching reference_scene (requires `Msaa::Off` + TAA).
pub fn mood_camera_bundle() -> impl Bundle {
    (
        Camera3d::default(),
        Hdr,
        Msaa::Off,
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: 0.18,
            ..Bloom::NATURAL
        },
        VolumetricFog {
            ambient_intensity: 0.0,
            step_count: 72,
            ..default()
        },
        ScreenSpaceAmbientOcclusion::default(),
        TemporalAntiAliasing::default(),
    )
}
