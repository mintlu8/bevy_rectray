use std::f32::consts::PI;

use bevy::{
    asset::{Assets, Handle},
    color::palettes::{basic::AQUA, css::GOLD},
    core_pipeline::bloom::BloomSettings,
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::{PbrBundle, StandardMaterial},
    prelude::{Camera3dBundle, SpatialBundle},
    window::{CursorIcon, PrimaryWindow, Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_app::{App, Startup};
use bevy_app::{PluginGroup, Update};
use bevy_ecs::{
    query::With,
    system::{Commands, Query, ResMut},
};
use bevy_hierarchy::BuildChildren;
use bevy_math::{primitives::Cuboid, Vec2, Vec3};
use bevy_mod_picking::{
    backends::raycast::RaycastBackendSettings, debug::DebugPickingMode, focus::PickingInteraction,
    DefaultPickingPlugins, PickableBundle,
};
use bevy_rectray::{Anchor, Dimension, RectrayBundle, RectrayFrame, RectrayPlugin, Transform2D};
use bevy_rectray_picking::{RectrayPickable, RectrayPickingBackendPlugin};
use bevy_render::{
    camera::Camera,
    mesh::{Mesh, Meshable},
};
use bevy_transform::components::Transform;

pub static LOREM_IPSUM: &str = r#"Lorem ipsum dolor sit amet, consectetur adipiscing elit. Praesent vehicula tortor sem, id egestas elit tincidunt eu. Etiam ante sem, accumsan ut felis fermentum, viverra lobortis nibh. Morbi neque lectus, venenatis vel luctus eu, ullamcorper et enim. In suscipit tempus nunc, sit amet sagittis ligula pharetra in. In lacinia felis in ullamcorper tempus. Praesent placerat ipsum dolor, et eleifend enim tincidunt eu. Duis laoreet, ante ut scelerisque eleifend, velit nulla mattis augue, id cursus dui enim et est. Fusce in nibh mauris. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Pellentesque tincidunt hendrerit sagittis. Suspendisse gravida quis purus a venenatis. Etiam ipsum velit, ultrices et auctor ac, pharetra vitae justo. Maecenas vulputate ligula et dui eleifend eleifend quis at neque. Integer facilisis enim ligula, eget scelerisque quam sodales non. Integer sed euismod massa. Nam auctor nec dolor ut condimentum."#;

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, init)
        .add_systems(Update, picking_cursor)
        .add_plugins(DefaultPickingPlugins)
        .insert_resource(DebugPickingMode::Normal)
        // This disables raycast backend.
        .insert_resource(RaycastBackendSettings {
            require_markers: true,
            ..Default::default()
        })
        .add_plugins(RectrayPlugin)
        .add_plugins(RectrayPickingBackendPlugin)
        .run();
}

pub fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10., 10., 10.)
                .looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
            camera: Camera {
                hdr: true,
                ..Default::default()
            },
            ..Default::default()
        },
        BloomSettings::NATURAL,
    ));

    let mat = StandardMaterial {
        base_color: GOLD.into(),
        unlit: true,
        ..Default::default()
    };

    commands
        .spawn((
            SpatialBundle::default(),
            RectrayFrame::from_anchor_dimension(Anchor::CENTER, Vec2::new(800., 500.)),
        ))
        .with_children(|builder| {
            for i in 0..16 {
                let angle = i as f32 / 16. * PI * 2.;
                builder
                    .spawn((
                        SpatialBundle::default(),
                        RectrayBundle {
                            transform_2d: Transform2D {
                                rotation: angle,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                    ))
                    .with_children(|builder| {
                        builder.spawn((
                            PbrBundle {
                                mesh: meshes.add(Cuboid::new(1.4, 0.4, 0.01).mesh()),
                                material: mats.add(mat.clone()),
                                ..Default::default()
                            },
                            RectrayBundle {
                                transform_2d: Transform2D {
                                    offset: Vec2::new(0., 4.),
                                    anchor: Anchor::CENTER,
                                    scale: Vec2::splat(fastrand::f32() * 0.4 + 0.8),
                                    ..Default::default()
                                },
                                dimension: Dimension(Vec2::new(1.4, 0.4)),
                                ..Default::default()
                            },
                            RectrayPickable,
                            PickableBundle::default(),
                        ));
                    });
            }
        });
}

pub fn picking_cursor(
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    mut query: Query<(&PickingInteraction, &Handle<StandardMaterial>)>,
) {
    let Ok(mut window) = window.get_single_mut() else {
        return;
    };

    let mut hovering = false;

    for (inter, mat) in query.iter_mut() {
        match inter {
            PickingInteraction::None => {
                let _ = mats.get_mut(mat).map(|x| x.base_color = (GOLD * 4.).into());
            }
            _ => {
                let _ = mats.get_mut(mat).map(|x| x.base_color = (AQUA * 4.).into());
                hovering = true;
            }
        }
    }
    if hovering {
        window.cursor.icon = CursorIcon::Pointer;
    } else {
        window.cursor.icon = CursorIcon::Default;
    }
}
