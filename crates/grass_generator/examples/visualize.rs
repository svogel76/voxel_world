//! Bevy preview of [`grass_generator::generate`].
//!
//! Spawns cross-quad clumps from the unified API. Simple vertex colors per
//! [`GrassVariant`] — no textures.
//!
//! Run:
//! ```text
//! cargo run -p grass_generator --example visualize
//! ```
//!
//! Controls (Bevy `FreeCamera`):
//! - WASD: move, Q/E: down/up, Shift: run, scroll: speed
//! - Right-click or M: capture mouse for look-around

use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};
use grass_generator::{generate, Area, GrassInstance, GrassParams, GrassVariant};
use glam::Vec2;

const PREVIEW_SEED: u64 = 42;
const CLUMP_WIDTH: f32 = 0.7;
const CLUMP_HEIGHT: f32 = 1.1;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, (setup_scene, spawn_help_text))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let area = Area {
        min: Vec2::new(-12.0, -12.0),
        max: Vec2::new(12.0, 12.0),
    };
    let params = GrassParams::default();
    let instances = generate(PREVIEW_SEED, area, &params);

    eprintln!(
        "preview: {} instances (area {:.0}x{:.0}, density {:.1})",
        instances.len(),
        area.width(),
        area.depth(),
        params.density,
    );

    let quad_mesh = meshes.add(Rectangle::new(CLUMP_WIDTH, CLUMP_HEIGHT));
    let grass_material = materials.add(variant_material(GrassVariant::Grass));
    let fern_material = materials.add(variant_material(GrassVariant::Fern));

    for instance in instances {
        spawn_cross_quad(
            &mut commands,
            &quad_mesh,
            &instance,
            &grass_material,
            &fern_material,
        );
    }

    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.28, 0.22, 0.14),
        perceptual_roughness: 1.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(1.0)).mesh().size(30.0, 30.0))),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, -0.01, 0.0),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 18.0, 8.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 6.0, 22.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        FreeCamera {
            sensitivity: 0.15,
            walk_speed: 6.0,
            run_speed: 18.0,
            friction: 20.0,
            ..default()
        },
    ));
}

fn variant_material(variant: GrassVariant) -> StandardMaterial {
    let base_color = match variant {
        GrassVariant::Grass => Color::srgb(0.28, 0.62, 0.18),
        GrassVariant::Fern => Color::srgb(0.16, 0.48, 0.14),
    };

    StandardMaterial {
        base_color,
        double_sided: true,
        perceptual_roughness: 0.95,
        ..default()
    }
}

fn spawn_cross_quad(
    commands: &mut Commands,
    quad_mesh: &Handle<Mesh>,
    instance: &GrassInstance,
    grass_material: &Handle<StandardMaterial>,
    fern_material: &Handle<StandardMaterial>,
) {
    let material = match instance.variant {
        GrassVariant::Grass => grass_material.clone(),
        GrassVariant::Fern => fern_material.clone(),
    };

    commands
        .spawn((
            Transform::from_translation(instance.position)
                .with_rotation(Quat::from_rotation_y(instance.rotation_y))
                .with_scale(Vec3::splat(instance.scale)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(quad_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::IDENTITY,
            ));
            parent.spawn((
                Mesh3d(quad_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            ));
        });
}

fn spawn_help_text(mut commands: Commands) {
    let params = GrassParams::default();
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            left: px(12),
            ..default()
        },
        children![Text::new(format!(
            "Grass preview: generate() API, seed {PREVIEW_SEED}, density {:.1}\n\
             Two cross quads per instance (static, Y-rotated) — Grass / Fern colors\n\
             FreeCamera: WASD move | Q/E up/down | Shift run | Scroll speed\n\
             Right-click or M: mouse look",
            params.density,
        ))],
    ));
}
