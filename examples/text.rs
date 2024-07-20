use std::iter::repeat;

use bevy::{
    color::palettes::css::GOLD,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{Camera2dBundle, SpatialBundle},
    text::{Text, Text2dBundle, TextLayoutInfo, TextStyle},
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_app::{App, Startup};
use bevy_app::{PluginGroup, Update};
use bevy_ecs::system::{Commands, Query};
use bevy_hierarchy::BuildChildren;
use bevy_math::Vec2;
use bevy_rectray::{
    layout::{Container, LayoutControl, LayoutObject, ParagraphLayout},
    Anchor, Dimension, RectrayBundle, RectrayContainerBundle, RectrayFrame, RectrayPlugin,
    Transform2D,
};
use itertools::Itertools;

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
        .add_systems(Update, sync_size)
        .add_plugins(RectrayPlugin)
        .run();
}

pub fn init(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn((
            SpatialBundle::default(),
            RectrayFrame::from_anchor_dimension(Anchor::CENTER, Vec2::new(800., 500.)),
        ))
        .with_children(|builder| {
            builder
                .spawn((
                    SpatialBundle::default(),
                    RectrayContainerBundle {
                        transform_2d: Transform2D::UNIT,
                        dimension: Dimension(Vec2::new(800., 500.)),
                        container: Container {
                            layout: LayoutObject::new(ParagraphLayout::PARAGRAPH),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ))
                .with_children(|builder| {
                    for word in LOREM_IPSUM.split(' ').interleave_shortest(repeat(" ")) {
                        builder.spawn((
                            Text2dBundle {
                                text: Text::from_section(
                                    word,
                                    TextStyle {
                                        color: GOLD.into(),
                                        font_size: 16.,
                                        ..Default::default()
                                    },
                                ),
                                ..Default::default()
                            },
                            RectrayBundle {
                                transform_2d: Transform2D {
                                    anchor: Anchor::TOP_LEFT,
                                    ..Default::default()
                                },
                                layout: if word == " " {
                                    LayoutControl::WhiteSpace
                                } else {
                                    LayoutControl::None
                                },
                                ..Default::default()
                            },
                        ));
                    }
                });
        });
}

pub fn sync_size(mut query: Query<(&TextLayoutInfo, &mut Dimension)>) {
    for (info, mut dimension) in query.iter_mut() {
        dimension.0 = info.logical_size;
    }
}
