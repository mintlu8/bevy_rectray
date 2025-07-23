use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

use bevy::ecs::entity::Entity;
use bevy::math::Vec2;
use bevy::reflect::std_traits::ReflectDefault;
use bevy::reflect::Reflect;
use downcast_rs::{impl_downcast, Downcast};

use super::{util::*, LayoutInfo, LayoutRange};

// asserts layout is object safe
const _: Option<Box<dyn Layout>> = None;

/// A layout that accepts a one dimensional sequence of widgets.
///
/// The `Container` is usually a dynamic sized widget,
/// meaning it will update its size based on the size occupied by its children.
/// You can parent it to an anchor of
/// a fixed sized widget for alignment.
pub trait Layout: Downcast + Debug + Send + Sync + 'static {
    /// Place sprites in the layout.
    fn place(
        &self,
        parent: &LayoutInfo,
        entities: Vec<LayoutItem>,
        range: &mut LayoutRange,
    ) -> LayoutOutput;
    /// Clone the layout.
    fn dyn_clone(&self) -> Box<dyn Layout>;
    /// Layout is the same regardless of parent dimension.
    fn is_size_agnostic(&self) -> bool {
        false
    }
}

impl_downcast!(Layout);

/// Type erased [`Layout`].
#[derive(Debug, Reflect)]
#[reflect(Default)]
pub struct LayoutObject(#[reflect(ignore)] Box<dyn Layout>);

impl Default for LayoutObject {
    fn default() -> Self {
        Self(Box::new(BoundsLayout::PADDING))
    }
}

impl LayoutObject {
    pub fn new(layout: impl Layout) -> Self {
        Self(Box::new(layout))
    }

    pub fn downcast_ref<T: Layout>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref()
    }

    pub fn downcast_mut<T: Layout>(&mut self) -> Option<&mut T> {
        self.0.as_any_mut().downcast_mut()
    }
}

impl Clone for LayoutObject {
    fn clone(&self) -> Self {
        Self(self.0.dyn_clone())
    }
}

impl<T> From<T> for LayoutObject
where
    T: Layout,
{
    fn from(value: T) -> Self {
        Self(Box::new(value))
    }
}

impl Deref for LayoutObject {
    type Target = dyn Layout;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

/// Output of a layout, containing anchors of entities, and the computed dimension of the layout.
#[derive(Debug)]
pub struct LayoutOutput {
    pub entity_anchors: Vec<(Entity, Vec2)>,
    pub dimension: Vec2,
    /// Maximum value for the layout.
    pub max_count: usize,
}

impl LayoutOutput {
    pub fn normalized(mut self) -> Self {
        self.entity_anchors
            .iter_mut()
            .for_each(|(_, x)| *x = *x / self.dimension - 0.5);
        self
    }
    pub fn with_max(mut self, max: usize) -> Self {
        self.max_count = max;
        self
    }
}

/// A dynamic dimensioned layout with size equal
/// to the maximum of its children and no additional behaviors.
#[derive(Debug, Clone, Copy, Reflect)]
pub struct BoundsLayout {
    /// If set, use `Dimension` on that axis.
    pub fixed: [bool; 2],
    /// Minimum bounds.
    pub min: Vec2,
    /// Maximum bounds.
    pub max: Vec2,
}

impl BoundsLayout {
    /// Ignore constraints and use `BoundsLayout` as padding.
    pub const PADDING: Self = Self {
        fixed: [false; 2],
        min: Vec2::ZERO,
        max: Vec2::MAX,
    };

    pub const fn from_max(max: Vec2) -> Self {
        BoundsLayout {
            fixed: [false; 2],
            min: Vec2::MAX,
            max,
        }
    }

    pub const fn from_min(min: Vec2) -> Self {
        BoundsLayout {
            fixed: [false; 2],
            min,
            max: Vec2::MAX,
        }
    }

    pub const fn x_bounds(min: f32, max: f32) -> Self {
        BoundsLayout {
            fixed: [false, true],
            min: Vec2::splat(min),
            max: Vec2::splat(max),
        }
    }

    pub const fn y_bounds(min: f32, max: f32) -> Self {
        BoundsLayout {
            fixed: [true, false],
            min: Vec2::splat(min),
            max: Vec2::splat(max),
        }
    }
}

impl Default for BoundsLayout {
    fn default() -> Self {
        Self::PADDING
    }
}

impl Layout for BoundsLayout {
    fn place(
        &self,
        info: &LayoutInfo,
        entities: Vec<LayoutItem>,
        range: &mut LayoutRange,
    ) -> LayoutOutput {
        let mut max_dim = Vec2::ZERO;
        range.resolve(entities.len());
        let entity_anchors: Vec<_> = entities[range.to_range(entities.len())]
            .iter()
            .map(|x| {
                max_dim = max_dim.max(x.dimension);
                (x.entity, x.anchor)
            })
            .collect();

        let min = self.min;
        let max = self.max;

        let dim = max_dim.clamp(min, max);

        let dimension = Vec2::new(
            if !self.fixed[0] {
                dim.x
            } else {
                info.dimension.x
            },
            if !self.fixed[1] {
                dim.y
            } else {
                info.dimension.y
            },
        );
        LayoutOutput {
            entity_anchors,
            dimension,
            max_count: entities.len(),
        }
    }

    fn dyn_clone(&self) -> Box<dyn Layout> {
        Box::new(*self)
    }
}

/// A size agnostic mono-directional container.
#[derive(Debug, Reflect)]
pub struct StackLayout<D: Direction = X>(#[reflect(ignore)] PhantomData<D>);

impl<D: Direction> Copy for StackLayout<D> {}
impl<D: Direction> Clone for StackLayout<D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl StackLayout {
    /// A left to right layout.
    pub const HSTACK: StackLayout<X> = StackLayout(PhantomData);
    /// A top to bottom layout.
    pub const VSTACK: StackLayout<Rev<Y>> = StackLayout(PhantomData);
}

impl<D: Direction> Default for StackLayout<D> {
    fn default() -> Self {
        StackLayout(PhantomData)
    }
}

impl<D: Direction> StackLayout<D> {
    pub fn new() -> Self {
        StackLayout(PhantomData)
    }
}

/// A fix-sized mono-directional container.
#[derive(Debug, Reflect)]
pub struct SpanLayout<D: StretchDir = X>(#[reflect(ignore)] PhantomData<D>);

impl<D: StretchDir> Copy for SpanLayout<D> {}
impl<D: StretchDir> Clone for SpanLayout<D> {
    fn clone(&self) -> Self {
        *self
    }
}

impl SpanLayout {
    /// A left to right layout with fixed dimension.
    pub const HBOX: SpanLayout<X> = SpanLayout(PhantomData);
    /// A top to bottom layout with fixed dimension.
    pub const VBOX: SpanLayout<Rev<Y>> = SpanLayout(PhantomData);
}

impl<D: StretchDir> Default for SpanLayout<D> {
    fn default() -> Self {
        SpanLayout(PhantomData)
    }
}

impl<D: StretchDir> SpanLayout<D> {
    pub fn new() -> Self {
        SpanLayout(PhantomData)
    }

    pub fn with_stretch(self) -> SpanLayout<Stretch<D>> {
        SpanLayout(PhantomData)
    }
}

/// A multiline version of the `span` layout, similar to the layout of a paragraph.
#[derive(Debug, Reflect)]
pub struct ParagraphLayout<D1: StretchDir = X, D2: Direction = Rev<Y>>(
    #[reflect(ignore)] PhantomData<(D1, D2)>,
)
where
    (D1, D2): DirectionPair;

impl<D1: StretchDir, D2: Direction> Copy for ParagraphLayout<D1, D2> where (D1, D2): DirectionPair {}
impl<D1: StretchDir, D2: Direction> Clone for ParagraphLayout<D1, D2>
where
    (D1, D2): DirectionPair,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl ParagraphLayout {
    /// A left to right, top to bottom paragraph, similar to the default layout of a webpage.
    pub const PARAGRAPH: Self = Self(PhantomData);
}

impl<D1: StretchDir, D2: Direction> Default for ParagraphLayout<D1, D2>
where
    (D1, D2): DirectionPair,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<D1: StretchDir, D2: Direction> ParagraphLayout<D1, D2>
where
    (D1, D2): DirectionPair,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn with_stretch(self) -> ParagraphLayout<Stretch<D1>, D2>
    where
        (Stretch<D1>, D2): DirectionPair,
    {
        ParagraphLayout::<Stretch<D1>, D2>(PhantomData)
    }
}
