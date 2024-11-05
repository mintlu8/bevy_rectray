use bevy::app::PluginGroup;
use bevy::app::{App, Startup};
use bevy::ecs::system::Commands;
use bevy::hierarchy::{BuildChildren, ChildBuilder};
use bevy::math::Vec2;
use bevy::{
    color::Color,
    diagnostic::FrameTimeDiagnosticsPlugin,
    prelude::{Camera2d, ChildBuild},
    sprite::Sprite,
    window::{Window, WindowPlugin},
    DefaultPlugins,
};
use bevy_rectray::{Anchor, Dimension, RectrayFrame, RectrayPlugin, Transform2D};

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
        .run();
}

pub fn init(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn((
            Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(500., 500.)),
                ..Default::default()
            },
            RectrayFrame::from_anchor_dimension(Anchor::CENTER, Vec2::new(500., 500.)),
        ))
        .with_children(|builder| {
            build_one(builder, Color::linear_rgb(1., 0., 0.), Anchor::TOP_LEFT);
            build_one(builder, Color::linear_rgb(1., 0.5, 0.), Anchor::TOP_CENTER);
            build_one(builder, Color::linear_rgb(1., 1., 0.), Anchor::TOP_RIGHT);
            build_one(builder, Color::linear_rgb(0., 1., 0.), Anchor::CENTER_LEFT);
            build_one(builder, Color::linear_rgb(0., 1., 1.), Anchor::CENTER);
            build_one(builder, Color::linear_rgb(0., 0., 1.), Anchor::CENTER_RIGHT);
            build_one(builder, Color::linear_rgb(0.5, 0., 1.), Anchor::BOTTOM_LEFT);
            build_one(
                builder,
                Color::linear_rgb(1., 0., 1.),
                Anchor::BOTTOM_CENTER,
            );
            build_one(
                builder,
                Color::linear_rgb(1., 0., 0.5),
                Anchor::BOTTOM_RIGHT,
            );
        });
}

fn build_one(builder: &mut ChildBuilder, color: Color, anchor: Anchor) {
    builder
        .spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::new(100., 50.)),
                ..Default::default()
            },
            Transform2D {
                anchor,
                center: anchor,
                ..Default::default()
            },
            Dimension(Vec2::new(100., 50.)),
        ))
        .with_children(|builder| {
            build_two(builder, Color::BLACK, Anchor::TOP_LEFT);
            build_two(builder, Color::BLACK, Anchor::TOP_CENTER);
            build_two(builder, Color::BLACK, Anchor::TOP_RIGHT);
            build_two(builder, Color::BLACK, Anchor::CENTER_LEFT);
            build_two(builder, Color::BLACK, Anchor::CENTER);
            build_two(builder, Color::BLACK, Anchor::CENTER_RIGHT);
            build_two(builder, Color::BLACK, Anchor::BOTTOM_LEFT);
            build_two(builder, Color::BLACK, Anchor::BOTTOM_CENTER);
            build_two(builder, Color::BLACK, Anchor::BOTTOM_RIGHT);
        });
}

fn build_two(builder: &mut ChildBuilder, color: Color, anchor: Anchor) {
    builder.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(15., 10.)),
            ..Default::default()
        },
        Transform2D {
            anchor,
            center: anchor,
            ..Default::default()
        },
        Dimension(Vec2::new(15., 10.)),
    ));
}
