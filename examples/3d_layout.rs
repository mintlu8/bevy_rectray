use std::collections::HashSet;

use bevy::app::PluginGroup;
use bevy::app::{App, Startup};
use bevy::ecs::system::{Commands, ResMut};
use bevy::hierarchy::BuildChildren;
use bevy::math::{
    primitives::{Cuboid, Cylinder, Plane3d, Sphere, Torus},
    Vec2, Vec3,
};
use bevy::transform::components::Transform;
use bevy::{
    asset::Assets,
    color::Color,
    diagnostic::FrameTimeDiagnosticsPlugin,
    image::Image,
    pbr::{AmbientLight, DirectionalLight, MeshMaterial3d, StandardMaterial},
    prelude::{Camera3d, ChildBuild, Mesh3d},
    render::{
        mesh::{Mesh, Meshable},
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_rectray::{
    layout::{Container, LayoutObject, StackLayout},
    Anchor, Dimension, RectrayFrame, RectrayPlugin, Transform2D,
};

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
        .add_plugins(RectrayPlugin)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1000.,
        })
        .run();
}

pub fn init(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(20., 20., 20.).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 8000.,
            ..Default::default()
        },
        Transform::from_xyz(10., -20., 20.).looking_at(Vec3::ZERO, Vec3::Z),
    ));

    let mat = mats.add(StandardMaterial {
        base_color_texture: Some(images.add(uv_debug_texture())),
        ..Default::default()
    });

    commands
        .spawn((
            RectrayFrame::from_anchor_dimension(Anchor::CENTER, Vec2::new(10., 10.)),
            Transform::from_xyz(0., 0., 8.),
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Transform2D::IDENTITY,
                    Dimension(Vec2::new(10., 10.)),
                    Container {
                        layout: LayoutObject::new(StackLayout::HSTACK),
                        margin: Vec2::new(0.1, 0.1),
                        ..Default::default()
                    },
                ))
                .with_children(|builder| {
                    for i in HashSet::<u32>::from_iter(0u32..10u32) {
                        let pos = Vec2::new(fastrand::f32() * 1.5 + 0.5, 1.);
                        builder.spawn((
                            Mesh3d(meshes.add(random_mesh(i).scaled_by(pos.extend(1.)))),
                            MeshMaterial3d(mat.clone()),
                            Transform2D {
                                anchor: Anchor::CENTER_LEFT,
                                ..Default::default()
                            },
                            Dimension(Vec2::new(2.0, 2.0) * pos),
                        ));
                    }
                });
        });
}

fn random_mesh(value: u32) -> Mesh {
    match value % 5 {
        0 => Sphere::new(1.).mesh().into(),
        1 => Cuboid::new(1.8, 1.8, 1.8).mesh().into(),
        2 => Torus {
            minor_radius: 0.25,
            major_radius: 0.75,
        }
        .into(),
        3 => Cylinder::new(1., 1.).mesh().into(),
        4 => Sphere::new(1.).mesh().ico(1).unwrap(),
        _ => Plane3d::new(Vec3::Y, Vec2::ONE).into(),
    }
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}
