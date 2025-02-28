use std::mem;

use bevy::ecs::{
    entity::Entity,
    system::{Local, Query},
};
use bevy::hierarchy::Children;
use bevy::transform::components::Transform;

use crate::rect::Transform2;
use crate::OutOfFrameBehavior;
use crate::{
    hierarchy::RectrayFrame,
    layout::{Container, LayoutControl, LayoutInfo, LayoutItem, LayoutOutput},
    rect::{ParentInfo, RotatedRect},
    transform::{Dimension, Transform2D},
};

type REntity<'t> = (
    Entity,
    &'t Dimension,
    &'t Transform2D,
    &'t OutOfFrameBehavior,
    &'t LayoutControl,
);

#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_ref_mut)]
fn propagate(
    parent: ParentInfo,
    entity: Entity,
    mut_query: &mut Query<REntity>,
    layout_query: &mut Query<&mut Container>,
    child_query: &Query<&Children>,
    queue: &mut Vec<(Entity, ParentInfo)>,
    transform_query: &mut Query<(&mut Transform, &mut RotatedRect)>,
) {
    if !mut_query.contains(entity) {
        return;
    }

    let Ok((entity, dim, transform, behavior, ..)) = mut_query.get(entity) else {
        return;
    };

    let dimension = dim.0;

    if let Ok(mut layout) = layout_query.get_mut(entity) {
        let children = child_query
            .get(entity)
            .map(|x| x.iter().copied())
            .into_iter()
            .flatten();
        let mut other_entities = Vec::new();
        let mut args = Vec::new();
        for child in children {
            if !mut_query.contains(child) {
                continue;
            }

            if let Ok((_, child_dim, child_transform, .., control)) = mut_query.get(child) {
                match control {
                    LayoutControl::IgnoreLayout => {
                        other_entities.push((child, child_transform.get_parent_anchor()))
                    }
                    control => {
                        args.push(LayoutItem {
                            entity: child,
                            anchor: child_transform.get_parent_anchor(),
                            dimension: child_dim.0,
                            control: *control,
                        });
                    }
                };
            }
        }
        let margin = layout.margin;
        let LayoutOutput {
            mut entity_anchors,
            dimension: new_dim,
            max_count,
        } = layout.place(&LayoutInfo { dimension, margin }, args);
        layout.maximum = max_count;
        let padding = layout.padding * 2.0;
        let fac = new_dim / (new_dim + padding);
        let size = new_dim + padding;
        if !fac.is_nan() {
            entity_anchors.iter_mut().for_each(|(_, anc)| *anc *= fac);
        }
        let rect = RotatedRect::construct(&parent, transform, size, parent.frame);

        let info = ParentInfo {
            dimension: new_dim,
            center: transform.get_center(),
            anchor: None,
            affine: parent
                .affine
                .mul(rect.transform2_at(transform.get_center())),
            frame: parent.frame,
            frame_rect: parent.frame_rect,
        };

        queue.extend(
            entity_anchors
                .into_iter()
                .map(|(e, anc)| (e, info.with_anchor(anc))),
        );
        if let Ok((mut t, mut r)) = transform_query.get_mut(entity) {
            *r = rect.under_transform2(parent.affine);
            *t = rect.transform_at(transform.get_center());
        }
        for (child, _) in other_entities {
            queue.push((child, info))
        }
        return;
    }

    let rect = match behavior {
        OutOfFrameBehavior::None => {
            RotatedRect::construct(&parent, transform, dimension, parent.frame)
        }
        OutOfFrameBehavior::Nudge => {
            let mut rect = RotatedRect::construct(&parent, transform, dimension, parent.frame);
            let frame_space_rect = rect.under_transform2(parent.affine);
            frame_space_rect.nudge_inside_ext(parent.frame_rect, &mut rect.center);
            rect
        }
        OutOfFrameBehavior::FlexAnchor { .. } => {
            let mut result = RotatedRect::construct(&parent, transform, dimension, parent.frame);
            for anchor in behavior.iter_flex_anchor() {
                let rect = RotatedRect::construct2(
                    &parent,
                    transform,
                    anchor.to_parent_anchor().into(),
                    anchor.to_anchor().into(),
                    dimension,
                    parent.frame,
                );
                let frame_space_rect = rect.under_transform2(parent.affine);
                if frame_space_rect.is_inside(parent.frame_rect) {
                    result = rect;
                }
            }
            result
        }
    };

    if let Ok(children) = child_query.get(entity) {
        let info = ParentInfo {
            dimension,
            anchor: None,
            center: transform.get_center(),
            affine: parent
                .affine
                .mul(rect.transform2_at(transform.get_center())),
            frame: parent.frame,
            frame_rect: parent.frame_rect,
        };
        for child in children.iter().copied() {
            queue.push((child, info))
        }
    }

    if let Ok((mut t, mut r)) = transform_query.get_mut(entity) {
        *t = rect.transform_at(transform.get_center());
        *r = rect.under_transform2(parent.affine);
    }
}

/// The main computation step.
pub fn compute_transform_2d(
    mut queue_a: Local<Vec<(Entity, ParentInfo)>>,
    mut queue_b: Local<Vec<(Entity, ParentInfo)>>,
    root_query: Query<(Entity, &RectrayFrame, &Children)>,
    mut entity_query: Query<REntity>,
    mut layout_query: Query<&mut Container>,
    child_query: Query<&Children>,
    mut transform_query: Query<(&mut Transform, &mut RotatedRect)>,
) {
    for (frame, root, children) in root_query.iter() {
        for child in children.iter().copied() {
            queue_a.push((
                child,
                ParentInfo {
                    dimension: root.dimension,
                    center: root.at,
                    anchor: None,
                    affine: Transform2::IDENTITY,
                    frame,
                    frame_rect: root.rect(),
                },
            ))
        }
    }

    while !queue_a.is_empty() {
        mem::swap::<Vec<_>>(queue_a.as_mut(), queue_b.as_mut());
        for (entity, parent) in queue_b.drain(..) {
            propagate(
                parent,
                entity,
                &mut entity_query,
                &mut layout_query,
                &child_query,
                &mut queue_a,
                &mut transform_query,
            );
        }
    }
}
