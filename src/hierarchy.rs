use bevy::ecs::{component::Component, reflect::ReflectComponent};
use bevy::math::{Rect, Vec2};
use bevy::prelude::{ReflectDefault, ReflectDeserialize, ReflectSerialize, Transform, Visibility};
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::rect::Anchor;

/// A root node that creates an area to place child entities.
#[derive(Debug, Default, Reflect, Component, Serialize, Deserialize)]
#[reflect(Component, Default, Serialize, Deserialize)]
#[require(Transform, Visibility)]
pub struct RectrayFrame {
    pub dimension: Vec2,
    pub at: Vec2,
    pub z: f32,
}

impl RectrayFrame {
    pub const fn from_dimension(dimension: Vec2) -> Self {
        Self {
            dimension,
            at: Vec2::ZERO,
            z: 0.0,
        }
    }

    pub const fn from_anchor_dimension(anchor: Anchor, dimension: Vec2) -> Self {
        Self {
            dimension,
            at: anchor.as_vec(),
            z: 0.0,
        }
    }

    pub const fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    pub fn rect(&self) -> Rect {
        let center = self.dimension * (-self.at);
        Rect {
            min: center - self.dimension / 2.0,
            max: center + self.dimension / 2.0,
        }
    }
}
