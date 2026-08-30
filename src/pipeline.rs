use std::mem;

use bevy::ecs::hierarchy::Children;
#[cfg(feature = "a11y")]
use bevy::ecs::query::With;
use bevy::transform::components::Transform;
use bevy::{
    ecs::{
        change_detection::DetectChanges,
        entity::Entity,
        system::{Local, Query, Res},
        world::Ref,
    },
    math::{Quat, StableInterpolate},
    time::{Time, Virtual},
};

use crate::OutOfFrameBehavior;
use crate::transform::{ChangeState, RectrayChangeDetection};
use crate::{
    hierarchy::RectrayFrame,
    layout::{Container, LayoutControl, LayoutInfo, LayoutItem, LayoutOutput},
    rect::{ParentInfo, RotatedRect},
    transform::{Dimension, Transform2D},
};
use crate::{rect::Transform2, transform::InterpolateTransform};

type REntity<'t> = (
    Entity,
    &'t Dimension,
    &'t Transform2D,
    &'t OutOfFrameBehavior,
    &'t LayoutControl,
    &'t RectrayChangeDetection,
);

fn exp_decay_interpolate(transform: &mut Transform, target: Transform, fac: f32, dt: f32) {
    transform
        .translation
        .smooth_nudge(&target.translation, fac, dt);
    transform.scale.smooth_nudge(&target.scale, fac, dt);
    let mut angle = transform.rotation.to_axis_angle().1;
    let to = target.rotation.to_axis_angle().1;
    angle.smooth_nudge(&to, fac, dt);
    transform.rotation = Quat::from_rotation_z(angle);
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::needless_pass_by_ref_mut)]
fn propagate(
    parent: ParentInfo,
    entity: Entity,
    dt: f32,
    mut_query: &mut Query<REntity>,
    layout_query: &mut Query<&mut Container>,
    child_query: &Query<&Children>,
    queue: &mut Vec<(Entity, ParentInfo)>,
    transform_query: &mut Query<(&mut Transform, &mut RotatedRect, Ref<InterpolateTransform>)>,
    #[cfg(feature = "a11y")] a11y_query: &mut Query<&mut bevy::a11y::AccessibilityNode>,
    #[cfg(feature = "a11y")] scale_factor: f32,
) {
    if !mut_query.contains(entity) {
        return;
    }

    let Ok((entity, dim, transform, behavior, .., change)) = mut_query.get(entity) else {
        return;
    };

    let skip_output = match change.state {
        ChangeState::Skip => return,
        ChangeState::Changed => false,
        ChangeState::Unchanged => true,
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

            if let Ok((_, child_dim, child_transform, .., control, _)) = mut_query.get(child) {
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
        if !skip_output && let Ok((mut t, mut r, interpolate)) = transform_query.get_mut(entity) {
            *r = rect.under_transform2(parent.affine);
            let result = rect.transform_at(transform.get_center());
            match &*interpolate {
                _ if interpolate.is_changed() => *t = result,
                InterpolateTransform::None => *t = result,
                InterpolateTransform::ExponentialDecay(fac) => {
                    exp_decay_interpolate(&mut t, result, *fac, dt);
                }
            }

            #[cfg(feature = "a11y")]
            if let Ok(mut node) = a11y_query.get_mut(entity)
                && rect.rotation == 0.
            {
                node.0
                    .set_bounds((*r).into_accesskit_rect(parent.frame_rect.size(), scale_factor));
            }
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
        OutOfFrameBehavior::AnchorSwap { .. } => {
            let mut result = RotatedRect::construct(&parent, transform, dimension, parent.frame);
            for anchor in behavior.iter_anchor_swaps() {
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
                    break;
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

    if !skip_output && let Ok((mut t, mut r, interpolate)) = transform_query.get_mut(entity) {
        *r = rect.under_transform2(parent.affine);
        let result = rect.transform_at(transform.get_center());
        match &*interpolate {
            _ if interpolate.is_changed() => *t = result,
            InterpolateTransform::None => *t = result,
            InterpolateTransform::ExponentialDecay(fac) => {
                exp_decay_interpolate(&mut t, result, *fac, dt);
            }
        }

        #[cfg(feature = "a11y")]
        if let Ok(mut node) = a11y_query.get_mut(entity)
            && rect.rotation == 0.
        {
            node.0
                .set_bounds((*r).into_accesskit_rect(parent.frame_rect.size(), scale_factor));
        }
    }
}

/// The main computation step.
pub fn compute_transform_2d(
    mut queue_a: Local<Vec<(Entity, ParentInfo)>>,
    mut queue_b: Local<Vec<(Entity, ParentInfo)>>,
    time: Res<Time<Virtual>>,
    root_query: Query<(Entity, &RectrayFrame, &Children)>,
    mut entity_query: Query<REntity>,
    mut layout_query: Query<&mut Container>,
    child_query: Query<&Children>,
    mut transform_query: Query<(&mut Transform, &mut RotatedRect, Ref<InterpolateTransform>)>,
    #[cfg(feature = "a11y")] mut a11y_query: Query<&mut bevy::a11y::AccessibilityNode>,
    #[cfg(feature = "a11y")] main_window: Query<
        &bevy::window::Window,
        With<bevy::window::PrimaryWindow>,
    >,
) {
    let dt = time.delta_secs();
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

    #[cfg(feature = "a11y")]
    let scale_factor = main_window.single().map(|x| x.scale_factor()).unwrap_or(1.);

    while !queue_a.is_empty() {
        mem::swap::<Vec<_>>(queue_a.as_mut(), queue_b.as_mut());
        for (entity, parent) in queue_b.drain(..) {
            propagate(
                parent,
                entity,
                dt,
                &mut entity_query,
                &mut layout_query,
                &child_query,
                &mut queue_a,
                &mut transform_query,
                #[cfg(feature = "a11y")]
                &mut a11y_query,
                #[cfg(feature = "a11y")]
                scale_factor,
            );
        }
    }
}

pub fn mark_dirty(
    root: Query<(Ref<RectrayFrame>, &Children)>,
    mut query: Query<(
        &mut RectrayChangeDetection,
        Ref<Transform2D>,
        Ref<Dimension>,
        Ref<OutOfFrameBehavior>,
        Option<Ref<Container>>,
    )>,
    children_query: Query<Ref<Children>>,
) {
    for (frame, children) in root.iter() {
        for child in children.iter().copied() {
            mark_dirty_on(child, frame.is_changed(), query.reborrow(), &children_query);
        }
    }
}

pub fn mark_dirty_on(
    entity: Entity,
    parent_is_dirty: bool,
    mut query: Query<(
        &mut RectrayChangeDetection,
        Ref<Transform2D>,
        Ref<Dimension>,
        Ref<OutOfFrameBehavior>,
        Option<Ref<Container>>,
    )>,
    children_query: &Query<Ref<Children>>,
) -> bool {
    let children = children_query.get(entity).ok();

    let mut any_children_changed = false;

    let Ok((_, transform, dimension, oofb, container)) = query.get_mut(entity) else {
        return true;
    };

    let oofb_dirty = match *oofb {
        OutOfFrameBehavior::None => false,
        _ => parent_is_dirty,
    };

    let self_dirty = transform.is_changed()
        || dimension.is_changed()
        || oofb.is_changed()
        || oofb_dirty
        || container.is_some();

    for child in children.iter().flatten().copied() {
        any_children_changed |= mark_dirty_on(child, self_dirty, query.reborrow(), children_query);
    }

    let Ok((mut change_detection, transform, dimension, oofb, container)) = query.get_mut(entity)
    else {
        return true;
    };

    let children_component_changed = children.is_some_and(|x| x.is_changed())
        || (children.is_none() && change_detection.had_children_before);

    let container_changed = container
        .is_some_and(|x| x.is_changed() || children_component_changed || any_children_changed);

    let changed = transform.is_changed()
        || dimension.is_changed()
        || oofb.is_changed()
        || oofb_dirty
        || container_changed;

    change_detection.had_children_before = children.is_some();
    change_detection.state = if changed {
        ChangeState::Changed
    } else if any_children_changed {
        ChangeState::Unchanged
    } else {
        ChangeState::Skip
    };
    if container.is_some() && changed {
        for child in children.iter().flatten().copied() {
            let Ok((mut change_detection, ..)) = query.get_mut(child) else {
                continue;
            };
            change_detection.state = ChangeState::Changed
        }
    }
    changed || any_children_changed
}
