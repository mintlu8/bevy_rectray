use std::iter::repeat;

use bevy::app::{App, Startup};
use bevy::app::{PluginGroup, Update};
use bevy::ecs::system::{Commands, Query};
use bevy::math::Vec2;
use bevy::{
    color::palettes::css::GOLD,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{Camera2d, Visibility},
    text::{Text2d, TextColor, TextFont, TextLayoutInfo},
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_rectray::{
    layout::{Container, LayoutControl, LayoutObject, ParagraphLayout},
    Anchor, Dimension, RectrayFrame, RectrayPlugin, Transform2D,
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
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, init)
        .add_systems(Update, sync_size)
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
                    Dimension(Vec2::new(800., 500.)),
                    Container {
                        layout: LayoutObject::new(ParagraphLayout::PARAGRAPH),
                        ..Default::default()
                    },
                    Visibility::Inherited,
                ))
                .with_children(|builder| {
                    for word in LOREM_IPSUM.split(' ').interleave_shortest(repeat(" ")) {
                        builder.spawn((
                            Text2d::new(word),
                            TextColor(GOLD.into()),
                            TextFont {
                                font_size: 16.,
                                ..Default::default()
                            },
                            Transform2D {
                                anchor: Anchor::TOP_LEFT,
                                ..Default::default()
                            },
                            if word == " " {
                                LayoutControl::WhiteSpace
                            } else {
                                LayoutControl::None
                            },
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
