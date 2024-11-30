use crate::layout::LayoutControl;
use crate::rect::Anchor;
use crate::RotatedRect;
use bevy::ecs::{component::Component, reflect::ReflectComponent};
use bevy::math::Vec2;
use bevy::prelude::{Transform, Visibility};
use bevy::reflect::{std_traits::ReflectDefault, Reflect, ReflectDeserialize, ReflectSerialize};
use serde::{Deserialize, Serialize};

/// The 2D transform component for `bevy_rectray`.
#[derive(Debug, Copy, Clone, Component, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize, Default)]
#[require(Transform, Visibility, Dimension, LayoutControl, RotatedRect)]
pub struct Transform2D {
    /// The anchor matched on the child side.
    ///
    /// If `offset` is 0, `anchor` on this rectangle and `parent_anchor` on the parent rectangle will overlap.
    pub anchor: Anchor,
    /// The anchor matched on the parent side.
    ///
    /// By default this is [`Anchor::INHERIT`],
    /// If set to `INHERIT`, would be the same as `anchor`.
    pub parent_anchor: Anchor,
    /// Position of outputted `Transform` as well as center of `rotation` and `scale`.
    ///
    /// By default this is [`Anchor::CENTER`],
    /// If set to `INHERIT`, would be the same as `anchor`.
    pub center: Anchor,
    /// Offset from parent's anchor.
    pub offset: Vec2,
    /// Z depth.
    /// By default this is `0.01`.
    pub z: f32,
    /// Rotation around `center`.
    pub rotation: f32,
    /// Scaling around `center`.
    pub scale: Vec2,
}

impl Transform2D {
    #[inline]
    pub fn get_center(&self) -> Vec2 {
        self.center.or(self.anchor).into()
    }

    #[inline]
    pub fn get_parent_anchor(&self) -> Vec2 {
        self.parent_anchor.or(self.anchor).into()
    }

    pub const UNIT: Self = Self {
        anchor: Anchor::CENTER,
        parent_anchor: Anchor::INHERIT,
        center: Anchor::CENTER,
        offset: Vec2::ZERO,
        rotation: 0.0,
        z: 0.01,
        scale: Vec2::ONE,
    };

    /// Set offset.
    #[inline]
    pub fn with_offset(mut self, offset: Vec2) -> Self {
        self.offset = offset;
        self
    }

    /// Set rotation.
    #[inline]
    pub fn with_rotation(mut self, rot: f32) -> Self {
        self.rotation = rot;
        self
    }

    /// Set scale.
    #[inline]
    pub fn with_scale(mut self, scale: Vec2) -> Self {
        self.scale = scale;
        self
    }

    /// Set z offset.
    #[inline]
    pub fn with_z(mut self, z: f32) -> Self {
        self.z = z;
        self
    }

    /// Set anchor.
    #[inline]
    pub fn with_anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Set parent anchor.
    #[inline]
    pub fn with_parent_anchor(mut self, anchor: Anchor) -> Self {
        self.parent_anchor = anchor;
        self
    }

    /// Set center.
    #[inline]
    pub fn with_center(mut self, center: Anchor) -> Self {
        self.center = center;
        self
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::UNIT
    }
}

/// Dimension of the widget, this is a suggestion and can be modified via `Layout`.
#[derive(Debug, Clone, Copy, PartialEq, Default, Component, Serialize, Deserialize, Reflect)]
#[reflect(Component, Serialize, Deserialize)]
pub struct Dimension(pub Vec2);

impl Dimension {
    pub const ZERO: Dimension = Dimension(Vec2::ZERO);
}
