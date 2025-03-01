use bevy::{
    ecs::{
        query::{Changed, With},
        system::Query,
    },
    prelude::{
        Component, Reflect, ReflectComponent, ReflectDefault, ReflectDeserialize, ReflectSerialize,
    },
    window::{PrimaryWindow, Window},
};
use serde::{Deserialize, Serialize};

use crate::{RectrayFrame, Transform2D};

/// Synchronize the size of [`RectrayFrame`] with [`PrimaryWindow`].
#[derive(Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize, Component)]
#[reflect(Default, Serialize, Deserialize, Component)]
pub struct RectrayWindow;

/// Set [`Transform2D::offset`] to [`PrimaryWindow`]'s cursor position.
#[derive(Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize, Component)]
#[reflect(Default, Serialize, Deserialize, Component)]
pub struct RectrayCursor;

pub fn window_frame_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut frames: Query<&mut RectrayFrame, With<RectrayWindow>>,
    mut cursors: Query<&mut Transform2D, With<RectrayCursor>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let size = window.size();
    for mut frame in &mut frames {
        frame.dimension = size;
    }
    if let Some(pos) = window.cursor_position() {
        for mut transform in &mut cursors {
            transform.offset = pos;
        }
    }
}

