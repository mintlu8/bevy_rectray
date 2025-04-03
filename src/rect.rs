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
        self.0.x.is_nan()
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
    /// Entity of the frame.
    pub frame_entity: Option<Entity>,
}

/// Relevant info about a parent.
#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
pub struct ParentInfo {
    pub dimension: Vec2,
    pub center: Vec2,
    pub anchor: Option<Vec2>,
    pub affine: Transform2,
    pub frame: Entity,
    pub frame_rect: Rect,
}

#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
pub struct Transform2 {
    pub translation: Vec2,
    pub rotation: f32,
    pub scale: Vec2,
}

impl Transform2 {
    pub const IDENTITY: Transform2 = Transform2 {
        translation: Vec2::ZERO,
        rotation: 0.,
        scale: Vec2::ONE,
    };

    pub fn mul(&self, other: Transform2) -> Self {
        Transform2 {
            translation: self.transform_point2(other.translation),
            rotation: self.rotation + other.rotation,
            scale: self.scale * other.scale,
        }
    }

    pub fn transform_point2(&self, point: Vec2) -> Vec2 {
        Vec2::from_angle(self.rotation).rotate(point) * self.scale + self.translation
    }
}

impl ParentInfo {
    pub fn with_anchor(mut self, anc: Vec2) -> Self {
        self.anchor = Some(anc);
        self
    }
}

impl RotatedRect {
    /// Find the frame space position of an anchor.
    #[inline]
    pub fn anchor(&self, anchor: Anchor) -> Vec2 {
        Vec2::from_angle(self.rotation).rotate(self.dimension * anchor) + self.center
    }

    // Half dimension
    #[inline]
    pub fn half_dim(&self) -> Vec2 {
        self.dimension / 2.
    }

    /// convert a frame space point to local space, centered on `Center`.
    #[inline]
    pub fn local_space(&self, position: Vec2) -> Vec2 {
        Vec2::from_angle(-self.rotation).rotate(position - self.center)
    }

    pub(crate) fn transform2_at(&self, center: Vec2) -> Transform2 {
        Transform2 {
            translation: self.anchor((-center).into()),
            rotation: self.rotation,
            scale: self.scale,
        }
    }

    pub fn transform_at(&self, center: Vec2) -> Transform {
        Transform {
            translation: self.anchor((-center).into()).extend(self.z),
            rotation: Quat::from_rotation_z(self.rotation),
            scale: self.scale.extend(1.0),
        }
    }

    pub(crate) fn under_transform2(mut self, transform: Transform2) -> Self {
        self.center = transform.transform_point2(self.center);
        self.rotation += transform.rotation;
        self.scale *= transform.scale;
        self
    }

    /// Create an [`RotatedRect`] representing the sprite's position in parent space.
    #[inline]
    pub fn construct(
        parent: &ParentInfo,
        transform: &Transform2D,
        dimension: Vec2,
        frame: Entity,
    ) -> Self {
        let parent_anchor = parent.anchor.unwrap_or(transform.get_parent_anchor());
        let anchor = transform.anchor.as_vec();
        Self::construct2(parent, transform, parent_anchor, anchor, dimension, frame)
    }

    /// Create an [`RotatedRect`] representing the sprite's position in parent space.
    #[inline]
    #[doc(hidden)]
    pub fn construct2(
        parent: &ParentInfo,
        transform: &Transform2D,
        parent_anchor: Vec2,
        anchor: Vec2,
        dimension: Vec2,
        frame: Entity,
    ) -> Self {
        let root = parent.dimension * parent_anchor;
        // apply offset and dimension
        let self_center = root + transform.offset + (transform.get_center() - anchor) * dimension;
        Self {
            center: self_center,
            dimension,
            z: transform.z,
            rotation: transform.rotation,
            scale: transform.scale,
            frame_entity: Some(frame),
        }
    }

    /// Determines if inside a [`Rect`].
    pub fn aabb(&self) -> Rect {
        let bl = self.anchor(Anchor::BOTTOM_LEFT);
        let tr = self.center * 2.0 - bl;
        Rect {
            min: bl.min(tr),
            max: bl.max(tr),
        }
    }

    /// Determines if inside a [`Rect`].
    pub fn is_inside(&self, rect: Rect) -> bool {
        let bl = self.anchor(Anchor::BOTTOM_LEFT);
        let tr = self.center * 2.0 - bl;
        rect.contains(bl) && rect.contains(tr)
    }

    /// Nudge the [`RotatedRect`] inside the [`Rect`],
    /// does nothing if aabb is larger than [`Rect`].
    pub fn nudge_inside(&mut self, bounds: Rect) {
        let aabb = self.aabb();
        if aabb.size().cmpgt(bounds.size()).any() {
            return;
        }
        nudge_aabb_with(&mut self.center, aabb, bounds);
    }

    /// Nudge the [`RotatedRect`] inside the [`Rect`],
    /// does nothing if aabb is larger than [`Rect`].
    pub(crate) fn nudge_inside_ext(&self, bounds: Rect, value: &mut Vec2) {
        let aabb = self.aabb();
        if aabb.size().cmpgt(bounds.size()).any() {
            return;
        }
        nudge_aabb_with(value, aabb, bounds);
    }
}

fn nudge_aabb_with(output: &mut Vec2, aabb: Rect, bounds: Rect) {
    if aabb.min.x < bounds.min.x {
        output.x += bounds.min.x - aabb.min.x;
    } else if aabb.max.x > bounds.max.x {
        output.x -= aabb.max.x - bounds.max.x;
    }

    if aabb.min.y < bounds.min.y {
        output.y += bounds.min.y - aabb.min.y;
    } else if aabb.max.y > bounds.max.y {
        output.y -= aabb.max.y - bounds.max.y;
    }
}
