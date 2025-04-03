use bevy::{
    asset::RenderAssetUsages,
    diagnostic::FrameTimeDiagnosticsPlugin,
    picking::hover::PickingInteraction,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::{PrimaryWindow, SystemCursorIcon},
    winit::cursor::CursorIcon,
};
use bevy_rectray::{
    layout::{Container, LayoutObject, StackLayout},
    Anchor, Dimension, InterpolateTransform, RectrayFrame, RectrayPickable, RectrayPlugin,
    Transform2D,
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
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, init)
        .add_systems(Update, pick)
        .add_plugins(RectrayPlugin)
        .run();
}

pub fn init(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(RectrayFrame::from_anchor_dimension(
            Anchor::CENTER,
            Vec2::new(10., 10.),
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Transform2D::IDENTITY.with_offset(Vec2::new(0., -20.)),
                    Dimension(Vec2::new(250., 25.)),
                    Container {
                        layout: LayoutObject::new(StackLayout::HSTACK),
                        margin: Vec2::new(20.0, 20.0),
                        ..Default::default()
                    },
                    Visibility::Inherited,
                ))
                .with_children(|builder| {
                    for _ in 0..9 {
                        let size = Vec2::new(fastrand::f32() * 80. + 5., 60.);
                        builder
                            .spawn((
                                Sprite {
                                    color: Color::hsl(fastrand::f32() * 360., 0.8, 0.5),
                                    custom_size: Some(size),
                                    ..Default::default()
                                },
                                Transform2D {
                                    ..Default::default()
                                },
                                Dimension(size),
                                RectrayPickable,
                                PickingInteraction::None,
                                InterpolateTransform::ExponentialDecay(5.),
                            ))
                            .observe(
                                |trigger: Trigger<Pointer<Pressed>>, mut commands: Commands| {
                                    println!("Entity {:?} goes BOOM!", trigger.target());
                                    commands.entity(trigger.target()).despawn();
                                },
                            );
                    }
                });
        });
}

pub fn pick(
    mut window: Query<&mut CursorIcon, With<PrimaryWindow>>,
    query: Query<&PickingInteraction, Changed<PickingInteraction>>,
) {
    let Ok(mut icon) = window.single_mut() else {
        return;
    };
    for interaction in &query {
        if *interaction != PickingInteraction::Pressed {
            *icon = CursorIcon::System(SystemCursorIcon::Grab);
            return;
        }
    }
    *icon = CursorIcon::System(SystemCursorIcon::Default);
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
