//! [`bevy_mod_picking`] backend for [`bevy_rectray`].
//!
//! # Getting Started
//!
//! Add `RectrayPickingBackendPlugin`.
//!
//! ```rust
//! # /*
//! app.add_plugins(RectrayPickingBackendPlugin)
//! # */
//! ```
//!
//! Add [`RectrayPickable`] and [`PickableBundle`](bevy_mod_picking::PickableBundle) to entities you want to be pickable, that's it!

#![allow(clippy::type_complexity)]
use bevy::ecs::{
    component::Component,
    entity::{Entity, EntityHashMap},
    event::EventWriter,
    query::With,
    system::{Query, Res},
};
use bevy::math::{primitives::InfinitePlane3d, Vec2, Vec3Swizzles};
use bevy::transform::components::GlobalTransform;
use bevy::{
    picking::backend::{ray::RayMap, HitData, PointerHits},
    prelude::Camera,
    render::view::RenderLayers,
};

use crate::{Dimension, RectrayFrame, RotatedRect, Transform2D};

/// Make an item pickable in the `bevy_rectray` backend.
///
/// Note: alternatives like the raycast backend or the sprite backend might be more desireable in some cases.
#[derive(Debug, Component, Default, Clone, Copy, PartialEq, Eq)]
#[require(Transform2D, Dimension)]
pub struct RectrayPickable;

/// System for the backed.
pub fn rectray_picking_backend(
    map: Res<RayMap>,
    layers: Query<(Option<&RenderLayers>, &Camera)>,
    frames: Query<&GlobalTransform, With<RectrayFrame>>,
    query: Query<(Entity, &RotatedRect, Option<&RenderLayers>), With<RectrayPickable>>,
    mut writer: EventWriter<PointerHits>,
) {
    let mut inverses = EntityHashMap::default();
    for (ray_id, ray) in map.iter() {
        let mut ray_hits = EntityHashMap::default();

        let Ok((layer, cam)) = layers.get(ray_id.camera) else {
            continue;
        };
        let cam_layer = if let Some(layer) = layer {
            layer
        } else {
            &RenderLayers::default()
        };
        let mut event = PointerHits {
            pointer: ray_id.pointer,
            picks: Vec::new(),
            order: cam.order as f32,
        };
        for (entity, rect, layers) in query.iter() {
            let layer = if let Some(layer) = layers {
                layer
            } else {
                &RenderLayers::default()
            };
            if !cam_layer.intersects(layer) {
                continue;
            }
            let Some(frame) = rect.frame_entity else {
                continue;
            };
            let ray_hit = ray_hits.entry(frame).or_insert_with(|| {
                let transform = frames.get(frame).ok()?;
                let inv = inverses
                    .entry(frame)
                    .or_insert_with(|| transform.affine().inverse());
                let plane = InfinitePlane3d::new(transform.forward());
                let depth = ray.intersect_plane(transform.translation(), plane)?;
                Some((
                    inv.transform_point3(ray.get_point(depth)),
                    depth,
                    transform.forward(),
                ))
            });
            let Some((ray_hit, depth, forward)) = *ray_hit else {
                continue;
            };
            let local = ray_hit.xy() - rect.center;
            let half_size = rect.dimension * rect.scale / 2.0;
            let inside = Vec2::from_angle(-rect.rotation)
                .rotate(local)
                .abs()
                .cmple(half_size)
                .all();
            if inside {
                event.picks.push((
                    entity,
                    HitData {
                        camera: ray_id.camera,
                        depth,
                        position: Some(ray_hit),
                        normal: Some(forward.into()),
                    },
                ))
            }
        }
        if !event.picks.is_empty() {
            writer.send(event);
        }
    }
}
