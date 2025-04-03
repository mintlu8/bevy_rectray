use bevy::app::{App, Startup};
use bevy::app::{PluginGroup, Update};
use bevy::ecs::{
    query::With,
    system::{Commands, Query},
};
use bevy::math::Vec2;
use bevy::picking::hover::PickingInteraction;
use bevy::prelude::Entity;
use bevy::{
    color::palettes::{basic::AQUA, css::GOLD},
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{Camera2d, Visibility},
    text::{Text2d, TextColor, TextFont, TextLayoutInfo},
    window::{PrimaryWindow, SystemCursorIcon, Window, WindowPlugin},
    winit::cursor::CursorIcon,
    DefaultPlugins,
};
use bevy_rectray::{
    layout::{Container, LayoutControl, LayoutObject, ParagraphLayout},
    Anchor, Dimension, RectrayFrame, RectrayPickable, RectrayPlugin, Transform2D,
};

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
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, init)
        .add_systems(Update, sync_size)
        .add_systems(Update, picking_cursor)
        .add_plugins(RectrayPlugin)
        .run();
}

pub fn init(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn(RectrayFrame::from_anchor_dimension(
            Anchor::CENTER,
            Vec2::new(800., 500.),
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    Transform2D::IDENTITY,
                    Dimension(Vec2::new(1024., 768.)),
                    Container {
                        layout: LayoutObject::new(ParagraphLayout::PARAGRAPH),
                        margin: Vec2::new(16., 16.),
                        ..Default::default()
                    },
                    Visibility::Inherited,
                ))
                .with_children(|builder| {
                    for word in LOREM_IPSUM.split(' ') {
                        builder.spawn((
                            Text2d::new(word),
                            TextColor(GOLD.into()),
                            TextFont {
                                font_size: 24.,
                                ..Default::default()
                            },
                            Transform2D {
                                anchor: Anchor::TOP_LEFT,
                                rotation: (fastrand::f32() - 0.5) * 0.5,
                                scale: Vec2::splat(fastrand::f32() * 0.4 + 0.8),
                                ..Default::default()
                            },
                            if word == " " {
                                LayoutControl::WhiteSpace
                            } else {
                                LayoutControl::None
                            },
                            PickingInteraction::None,
                            RectrayPickable,
                        ));
                    }
                });
        });
}

pub fn sync_size(mut query: Query<(&TextLayoutInfo, &mut Dimension)>) {
    for (info, mut dimension) in query.iter_mut() {
        dimension.0 = info.size;
    }
}

pub fn picking_cursor(
    mut commands: Commands,
    window: Query<Entity, With<PrimaryWindow>>,
    mut query: Query<(&PickingInteraction, &mut TextColor)>,
) {
    let Ok(window) = window.single() else {
        return;
    };

    let mut hovering = false;

    for (inter, mut text) in query.iter_mut() {
        match inter {
            PickingInteraction::None => {
                text.0 = GOLD.into();
            }
            _ => {
                text.0 = AQUA.into();
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
