#![allow(clippy::too_many_arguments)]
#![allow(clippy::type_complexity)]
//! A minimal 2d layout system (that works in 3d!) for bevy.
//!
//! # Getting Started
//!
//! First add `RectrayPlugin`.
//!
//! ```
//! # /*
//! app.add_plugins(RectrayPlugin)
//! # */
//! ```
//!
//! Then add [`RectrayFrame`] to a parent entity.
//! This effectively creates a 2d rectangular space around
//! the local `x` and `y` axis of the entity's `Transform`.
//!
//! ```
//! # /*
//! commands.spawn(
//!     SpacialBundle {
//!         ...
//!     },
//!     RectrayFrame::from_dimension(Vec2::new(1024., 768.)),
//! )
//! # */
//! ```
//!
//! To place descendant entities inside the frame, add [`Transform2D`] and [`Dimension`] next to entities
//! with [`Transform`](bevy::prelude::Transform)s.
//!
//!
//! ```
//! # /*
//! commands.spawn (
//!     Transform { .. },
//!     Transform2D { .. },
//!     Dimension { .. },
//! )
//! # */
//! ```
//!
//! Since we only operate on `Transform`, `bevy_rectray`
//! works in `Transform - Transform2d - Transform` sandwich situations.
//!
//!
//! # Integration
//!
//! `bevy_rectray` is minimal and does not magically react to changes in bevy components.
//! We take in [`Transform2D`] and [`Dimension`] and produces [`Transform`](bevy_transform::prelude::Transform)
//! and [`RotatedRect`].
//!
//! Some of those data can come from external sources.
//! For example if you want to make all `Sprite`s take up space of its `Image` or `custom_size`,
//! add a system like this manually:
//!
//! ```
//! # use bevy::{prelude::*, window::PrimaryWindow};
//! # use bevy_rectray::*;
//! pub fn update_sprite_dimension(
//!     scaling_factor: Query<&Window, With<PrimaryWindow>>,
//!     mut query: Query<(&mut Sprite, &Handle<Image>, &mut Dimension)>,
//!     assets: Res<Assets<Image>>
//! ) {
//!     let scaling_factor = scaling_factor
//!          .get_single()
//!          .map(|x| x.scale_factor())
//!          .unwrap_or(1.0);
//!     query.iter_mut().for_each(|(sp, im, mut dimension)| {
//!         dimension.0 = sp.custom_size.or_else(|| {
//!             sp.rect.map(|rect| (rect.max - rect.min) * scaling_factor)
//!                 .or_else(|| {
//!                     assets.get(im)
//!                         .map(|x|x.size().as_vec2() * scaling_factor)
//!                 })
//!         }).unwrap_or(Vec2::ZERO)
//!     })
//! }
//! ```
//!
//! If you want the opposite behavior, you can update the size of a sprite from
//! the outputted [`RotatedRect::dimension`].
//!
//! # Containers
//!
//! Add [`RectrayContainerBundle`] to put child items in a [`Layout`](crate::layout::Layout).
//!
//! See [module](crate::layout) level documentation for details.
//!

use bevy::app::{App, Plugin, PostUpdate, PreUpdate};
use bevy::ecs::schedule::{IntoSystemConfigs, IntoSystemSetConfigs, SystemSet};
use bevy::transform::TransformSystem;
use layout::{Container, LayoutControl};

mod hierarchy;

pub mod layout;
mod picking;
mod pipeline;
mod rect;
mod transform;

pub use hierarchy::*;
use picking::rectray_picking_backend;
pub use picking::RectrayPickable;
pub use pipeline::compute_transform_2d;
pub use rect::{Anchor, RotatedRect};
pub use transform::{Dimension, Transform2D};
/// [`Plugin`] for `bevy_rectray`.
#[derive(Debug, Clone, Copy)]
pub struct RectrayPlugin;

/// [`SystemSet`] for `bevy_rectray`, runs in [`PostUpdate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct RectrayTransformSet;

impl Plugin for RectrayPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Transform2D>();
        app.register_type::<Dimension>();
        app.register_type::<Container>();
        app.register_type::<RotatedRect>();
        app.register_type::<LayoutControl>();
        app.configure_sets(
            PostUpdate,
            RectrayTransformSet.before(TransformSystem::TransformPropagate),
        );
        app.add_systems(PreUpdate, rectray_picking_backend);
        app.add_systems(PostUpdate, compute_transform_2d.in_set(RectrayTransformSet));
    }
}

#[allow(deprecated)]
mod bundles {
    use bevy::prelude::Bundle;

    use crate::{
        layout::{Container, LayoutControl},
        Dimension, RotatedRect, Transform2D,
    };

    /// [`Bundle`] for `bevy_rectray`'s features, must be paired with a
    /// `TransformBundle`.
    #[derive(Debug, Default, Bundle)]
    #[deprecated = "Add `Transform2D` and `Dimension` (optional) directly."]
    pub struct RectrayBundle {
        /// Transform of the item.
        pub transform_2d: Transform2D,
        /// Dimension of the item.
        pub dimension: Dimension,
        /// Controls special behavior regarding layouts.
        pub layout: LayoutControl,
        /// This is an output node and can be left as `default`.
        pub rotated_rect: RotatedRect,
    }

    /// [`Bundle`] for `bevy_rectray`'s features and a container, must be paired with a
    /// `TransformBundle`.
    #[derive(Debug, Default, Bundle)]
    #[deprecated = "Add `Transform2D`, `Dimension` and `Container` directly."]
    pub struct RectrayContainerBundle {
        /// Transform of the item.
        pub transform_2d: Transform2D,
        /// Dimension of the item.
        pub dimension: Dimension,
        /// Container of the layout.
        pub container: Container,
        /// Controls special behavior regarding layouts.
        pub layout: LayoutControl,
        /// This is an output node and can be left as `default`.
        pub rotated_rect: RotatedRect,
    }
}

#[allow(deprecated)]
pub use bundles::{RectrayBundle, RectrayContainerBundle};
