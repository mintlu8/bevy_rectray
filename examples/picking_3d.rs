use std::f32::consts::PI;

use bevy::app::{App, Startup};
use bevy::app::{PluginGroup, Update};
use bevy::ecs::{
    query::With,
    system::{Commands, Query, ResMut},
};
use bevy::hierarchy::BuildChildren;
use bevy::math::{primitives::Cuboid, Vec2, Vec3};
use bevy::prelude::Entity;
use bevy::render::{
    camera::Camera,
    mesh::{Mesh, Meshable},
};
use bevy::transform::components::Transform;
use bevy::{
    asset::Assets,
    color::palettes::{basic::AQUA, css::GOLD},
    core_pipeline::bloom::Bloom,
    diagnostic::FrameTimeDiagnosticsPlugin,
    pbr::{MeshMaterial3d, StandardMaterial},
    picking::focus::PickingInteraction,
    prelude::{Camera3d, ChildBuild, Mesh3d, MeshPickingSettings, Visibility},
    window::{PrimaryWindow, SystemCursorIcon, Window, WindowPlugin},
    winit::cursor::CursorIcon,
    DefaultPlugins,
};
use bevy_rectray::{Anchor, Dimension, RectrayFrame, RectrayPickable, RectrayPlugin, Transform2D};

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
        // This disables raycast backend.
        .insert_resource(MeshPickingSettings {
            require_markers: true,
            ..Default::default()
        })
        .add_plugins(RectrayPlugin)
        .run();
}

pub fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Transform::from_xyz(10., 10., 10.).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        Camera {
            hdr: true,
            ..Default::default()
        },
        Camera3d::default(),
        Bloom::NATURAL,
    ));

    let mat = StandardMaterial {
        base_color: GOLD.into(),
        unlit: true,
        ..Default::default()
    };

    commands
        .spawn(RectrayFrame::from_anchor_dimension(
            Anchor::CENTER,
            Vec2::new(800., 500.),
        ))
        .with_children(|builder| {
            for i in 0..16 {
                let angle = i as f32 / 16. * PI * 2.;
                builder
                    .spawn((
                        Transform2D {
                            rotation: angle,
                            ..Default::default()
                        },
                        Visibility::Visible,
                    ))
                    .with_children(|builder| {
                        builder.spawn((
                            Mesh3d(meshes.add(Cuboid::new(1.4, 0.4, 0.01).mesh())),
                            MeshMaterial3d(mats.add(mat.clone())),
                            Transform2D {
                                offset: Vec2::new(0., 4.),
                                anchor: Anchor::CENTER,
                                scale: Vec2::splat(fastrand::f32() * 0.4 + 0.8),
                                ..Default::default()
                            },
                            Dimension(Vec2::new(1.4, 0.4)),
                            RectrayPickable,
                            PickingInteraction::None,
                        ));
                    });
            }
        });
}

pub fn picking_cursor(
    mut commands: Commands,
    mut mats: ResMut<Assets<StandardMaterial>>,
    window: Query<Entity, With<PrimaryWindow>>,
    mut query: Query<(&PickingInteraction, &MeshMaterial3d<StandardMaterial>)>,
) {
    let Ok(window) = window.get_single() else {
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
        commands
            .entity(window)
            .insert(CursorIcon::System(SystemCursorIcon::Pointer));
    } else {
        commands
            .entity(window)
            .insert(CursorIcon::System(SystemCursorIcon::Default));
    }
}
