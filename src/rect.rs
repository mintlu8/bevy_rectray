use std::ops::{Mul, Neg};

use bevy::ecs::entity::Entity;
use bevy::ecs::{component::Component, reflect::ReflectComponent};
use bevy::math::{Quat, Rect, Vec2};
use bevy::reflect::{std_traits::ReflectDefault, Reflect, ReflectDeserialize, ReflectSerialize};
use bevy::transform::components::Transform;
use serde::{Deserialize, Serialize};

use crate::Transform2D;

/// Anchor of a sprite, this is a more concise implementation than bevy's.
///
/// If a field is `Inherit` it will use `anchor` if possible.
#[derive(Debug, Clone, Copy, Default, PartialEq, Reflect, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Anchor(Vec2);

impl Anchor {
    pub const INHERIT: Self = Self(Vec2::NAN);
    pub const BOTTOM_LEFT: Self = Self(Vec2::new(-0.5, -0.5));
    pub const BOTTOM_CENTER: Self = Self(Vec2::new(0.0, -0.5));
    pub const BOTTOM_RIGHT: Self = Self(Vec2::new(0.5, -0.5));
    pub const CENTER_LEFT: Self = Self(Vec2::new(-0.5, 0.0));
    pub const CENTER: Self = Self(Vec2::ZERO);
    pub const CENTER_RIGHT: Self = Self(Vec2::new(0.5, 0.0));
    pub const TOP_LEFT: Self = Self(Vec2::new(-0.5, 0.5));
    pub const TOP_CENTER: Self = Self(Vec2::new(0.0, 0.5));
    pub const TOP_RIGHT: Self = Self(Vec2::new(0.5, 0.5));

    pub const fn new(v: Vec2) -> Self {
        Self(v)
    }

    pub const fn custom(x: f32, y: f32) -> Self {
        Self(Vec2::new(x, y))
    }

    pub fn is_inherit(&self) -> bool {
        self.0.is_nan()
    }

    pub const fn as_vec(&self) -> Vec2 {
        self.0
    }

    pub fn as_unit(&self) -> Vec2 {
        self.0 + Vec2::new(0.5, 0.5)
    }

    pub const fn x(&self) -> f32 {
        self.0.x
    }

    pub const fn y(&self) -> f32 {
        self.0.y
    }

    pub fn or(self, other: Self) -> Self {
        if self.is_inherit() {
            other
        } else {
            self
        }
    }

    pub fn str_name(&self) -> &'static str {
        match (self.0.x, self.0.y) {
            x if x.0.is_nan() || x.1.is_nan() => "Inherit",
            (x, y) if x < -0.16 && y < -0.16 => "BottomLeft",
            (x, y) if x < -0.16 && y > 0.16 => "TopLeft",
            (x, _) if x < -0.16 => "CenterLeft",
            (x, y) if x > 0.16 && y < -0.16 => "BottomRight",
            (x, y) if x > 0.16 && y > 0.16 => "TopRight",
            (x, _) if x > 0.16 => "CenterRight",
            (_, y) if y < -0.16 => "BottomCenter",
            (_, y) if y > 0.16 => "TopCenter",
            _ => "Center",
        }
    }
}

impl Neg for Anchor {
    type Output = Anchor;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl Mul<Vec2> for Anchor {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        self.0 * rhs
    }
}

impl Mul<Anchor> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Anchor) -> Self::Output {
        self * rhs.0
    }
}

impl From<Anchor> for Vec2 {
    fn from(val: Anchor) -> Self {
        val.0
    }
}

impl From<Vec2> for Anchor {
    fn from(val: Vec2) -> Self {
        Anchor(val)
    }
}

/// A rotated 2D rectangle.
///
/// Note: `scale` is independent from dimension.
#[derive(Debug, Clone, Copy, Component, PartialEq, Default, Serialize, Deserialize, Reflect)]
#[reflect(Component, Default, Serialize, Deserialize)]
pub struct RotatedRect {
    /// Center of the rect.
    pub center: Vec2,
    /// Size of the rect.
    pub dimension: Vec2,
    /// Rotation of the Rect.
    pub rotation: f32,
    /// Z depth of the Rect.
    pub z: f32,
    /// Scale of the rect.
    pub scale: Vec2,
}

/// Relevant info about a parent.
#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
pub struct ParentInfo {
    pub dimension: Vec2,
    pub center: Vec2,
    pub anchor: Option<Vec2>,
}

impl ParentInfo {
    pub fn with_anchor(mut self, anc: Vec2) -> Self {
        self.anchor = Some(anc);
        self
    }
}

impl RotatedRect {
    pub fn rect(&self) -> Rect {
        Rect {
            min: self.center - self.dimension / 2.,
            max: self.center + self.dimension / 2.,
        }
    }

    /// Find the screen space position of an anchor.
    #[inline]
    pub fn anchor(&self, anchor: Anchor) -> Vec2 {
        Vec2::from_angle(self.rotation).rotate(self.dimension * anchor) + self.center
    }

    // Half dimension
    #[inline]
    pub fn half_dim(&self) -> Vec2 {
        self.dimension / 2.
    }

    /// convert a screen space point to local space, centered on `Center`.
    #[inline]
    pub fn local_space(&self, position: Vec2) -> Vec2 {
        Vec2::from_angle(-self.rotation).rotate(position - self.center)
    }

    pub fn transform_at(&self, center: Vec2) -> Transform {
        Transform {
            translation: self.anchor((-center).into()).extend(self.z),
            rotation: Quat::from_rotation_z(self.rotation),
            scale: self.scale.extend(1.0),
        }
    }

    /// Create an [`RotatedRect`] representing the sprite's position in parent space.
    #[inline]
    pub fn construct(parent: &ParentInfo, transform: &Transform2D, dimension: Vec2) -> Self {
        let parent_anchor = parent.anchor.unwrap_or(transform.get_parent_anchor());
        let root = parent.dimension * parent_anchor;
        // apply offset and dimension
        let self_center = root
            + transform.offset
            + (transform.get_center() - transform.anchor.as_vec()) * dimension;
        Self {
            center: self_center,
            dimension,
            z: transform.z,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }
}

#[derive(Debug)]
pub struct FrameReference(pub Entity);
