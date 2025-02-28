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

use crate::RectrayFrame;

#[derive(Debug, Clone, Copy, Default, Reflect, Serialize, Deserialize, Component)]
#[reflect(Default, Serialize, Deserialize, Component)]
pub struct RectrayWindow;

pub fn window_frame_system(
    windows: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut frames: Query<&mut RectrayFrame, With<RectrayWindow>>,
) {
    let Ok(size) = windows.get_single().map(|x| x.size()) else {
        return;
    };
    for mut frame in &mut frames {
        frame.dimension = size;
    }
}
