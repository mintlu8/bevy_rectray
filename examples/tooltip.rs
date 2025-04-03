use bevy::sprite::Anchor as BevyAnchor;
use bevy::{
    color::palettes::css,
    diagnostic::FrameTimeDiagnosticsPlugin,
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
    text::TextBounds,
};
use bevy_rectray::{
    Anchor, AnchorDirection, Dimension, OutOfFrameBehavior, RectrayCursor, RectrayFrame,
    RectrayPlugin, RectrayWindow, SyncDimension, Transform2D,
};
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_systems(Startup, init)
        .add_systems(Update, mode_switch)
        .add_plugins(RectrayPlugin)
        .run();
}

#[derive(Debug, Component)]
pub struct Tooltip;

pub fn init(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands
        .spawn((RectrayFrame::default(), RectrayWindow))
        .with_children(|builder| {
            builder
                .spawn((
                    RectrayCursor,
                    Sprite {
                        color: css::RED.into(),
                        ..Default::default()
                    },
                    SyncDimension::FromDimension,
                    Transform2D::IDENTITY,
                ))
                .with_children(|builder| {
                    builder
                        .spawn((
                            Sprite {
                                color: css::GOLD.into(),
                                custom_size: Some(Vec2::new(200., 100.)),
                                ..Default::default()
                            },
                            SyncDimension::ToDimension,
                            Transform2D {
                                anchor: Anchor::TOP_LEFT,
                                ..Default::default()
                            },
                            OutOfFrameBehavior::Nudge,
                            Tooltip,
                        ))
                        .with_children(|builder| {
                            builder.spawn((
                                Text2d::new(
                                    "Press 'space' to switch between nudge and anchor swap mode.",
                                ),
                                TextBounds::new(200., 100.),
                                BevyAnchor::TopLeft,
                                TextColor(css::RED.into()),
                                Transform2D {
                                    anchor: Anchor::TOP_LEFT,
                                    ..Default::default()
                                },
                            ));
                        });
                });
        });
}

pub fn mode_switch(
    mut events: EventReader<KeyboardInput>,
    mut cursor: Single<&mut Dimension, With<RectrayCursor>>,
    mut tooltip: Single<&mut OutOfFrameBehavior, With<Tooltip>>,
) {
    for press in events.read() {
        if press.state == ButtonState::Pressed && press.key_code == KeyCode::Space {
            match &**tooltip {
                OutOfFrameBehavior::AnchorSwap { .. } => {
                    cursor.0 = Vec2::new(0., 0.);
                    **tooltip = OutOfFrameBehavior::Nudge;
                }
                _ => {
                    cursor.0 = Vec2::new(20., 20.);
                    **tooltip = OutOfFrameBehavior::anchor_swap(&[
                        AnchorDirection::RB,
                        AnchorDirection::LB,
                        AnchorDirection::RT,
                        AnchorDirection::LT,
                    ])
                }
            }
        }
    }
}
