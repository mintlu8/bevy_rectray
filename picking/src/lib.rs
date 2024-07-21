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
use bevy_app::{Plugin, PreUpdate};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::EventWriter,
    query::With,
    system::{Query, Res},
};
use bevy_math::{primitives::InfinitePlane3d, Vec2, Vec3Swizzles};
use bevy_mod_picking::backend::{ray::RayMap, HitData, PointerHits};
use bevy_rectray::{RotatedRect, Transform2D};
use bevy_render::{camera::Camera, view::RenderLayers};
use bevy_transform::components::GlobalTransform;

/// Make an item pickable in the `bevy_rectray` backend.
///
/// Note: alternatives like the raycast backend or the sprite backend might be more desireable in some cases.
#[derive(Debug, Component, Default, Clone, Copy, PartialEq, Eq)]
pub struct RectrayPickable;

/// System for the backed.
pub fn rectray_picking_backend(
    map: Res<RayMap>,
    layers: Query<(Option<&RenderLayers>, &Camera)>,
    query: Query<
        (
            Entity,
            &RotatedRect,
            &GlobalTransform,
            &Transform2D,
            Option<&RenderLayers>,
        ),
        With<RectrayPickable>,
    >,
    mut writer: EventWriter<PointerHits>,
) {
    for (ray_id, ray) in map.iter() {
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
        for (entity, rect, transform, transform_2d, layers) in query.iter() {
            let layer = if let Some(layer) = layers {
                layer
            } else {
                &RenderLayers::default()
            };
            if !cam_layer.intersects(layer) {
                continue;
            }
            let plane = InfinitePlane3d::new(transform.forward());
            let Some(depth) = ray.intersect_plane(transform.translation(), plane) else {
                continue;
            };
            let position = ray.get_point(depth);
            let local = transform.affine().inverse().transform_point3(position);
            let local = local.xy() - rect.dimension * transform_2d.center;
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
                        position: Some(position),
                        normal: Some(transform.forward().into()),
                    },
                ))
            }
        }
        if !event.picks.is_empty() {
            writer.send(event);
        }
    }
}

/// Plugin for adding a [`bevy_mod_picking`] backed for [`bevy_rectray`].
pub struct RectrayPickingBackendPlugin;

impl Plugin for RectrayPickingBackendPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        app.add_systems(PreUpdate, rectray_picking_backend);
    }
}
