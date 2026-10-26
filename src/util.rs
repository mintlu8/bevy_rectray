use std::{any::type_name, marker::PhantomData};

use bevy::{
    app::App, ecs::{
        entity::Entity,
        hierarchy::{ChildOf, Children},
        relationship::RelationshipTarget,
        resource::Resource,
        system::Commands,
        world::{EntityWorldMut, World},
    }, log::{error, warn},
};

pub trait ExchangeBuilder {
    fn root(&self, world: &World) -> Option<Entity>;

    fn invoke_for_each_removed(&mut self, entity: EntityWorldMut);

    fn invoke_for_each_added(&mut self, entity: EntityWorldMut);

    fn for_each_removed(self, f: impl FnMut(EntityWorldMut)) -> impl ExchangeBuilder
    where
        Self: Sized,
    {
        ForEachRemoved { builder: self, f }
    }

    fn for_each_added(self, f: impl FnMut(EntityWorldMut)) -> impl ExchangeBuilder
    where
        Self: Sized,
    {
        ForEachAdded { builder: self, f }
    }
}

pub trait ExchangeChildrenExt {
    /// Set children with a list, exchange discarded children with [`ExchangeRoot`].
    fn exchange_children(
        &mut self,
        entity: Entity,
        incoming: impl IntoIterator<Item = Entity>,
        arguments: impl ExchangeBuilder,
    );
}

pub trait ExchangeChildrenCommandExt {
    /// Set children with a list, exchange discarded children with [`ExchangeRoot`].
    fn exchange_children(
        &mut self,
        entity: Entity,
        incoming: impl IntoIterator<Item = Entity>,
        arguments: impl ExchangeBuilder + Send + Sync + 'static,
    );
}

#[derive(Resource)]
pub struct PhantomRes;

/// Argument for [`exchange_children`](ExchangeChildrenExt::exchange_children).
pub enum ExchangeRoot<R: Resource = PhantomRes> {
    /// Free removed children as root nodes.
    Root,
    /// Make removed children child of a specific entity.
    Entity(Entity),
    /// Make removed children child of a resource entity.
    Resource(PhantomData<R>),
}

impl ExchangeRoot {
    /// Make removed children child of a resource entity.
    pub fn resource<R: Resource>() -> ExchangeRoot<R> {
        ExchangeRoot::Resource(PhantomData)
    }
}

impl<R: Resource> ExchangeBuilder for ExchangeRoot<R> {
    fn root(&self, world: &World) -> Option<Entity> {
        match self {
            ExchangeRoot::Root => None,
            ExchangeRoot::Entity(entity) => Some(*entity),
            ExchangeRoot::Resource(_) => match world.resource_entity::<R>() {
                Some(e) => Some(e),
                None => {
                    error!(
                        "In `exchange_children`, resource `{}` missing.",
                        type_name::<R>()
                    );
                    None
                }
            },
        }
    }

    fn invoke_for_each_removed(&mut self, _entity: EntityWorldMut) {}

    fn invoke_for_each_added(&mut self, _entity: EntityWorldMut) {}
}

pub struct ForEachRemoved<B: ExchangeBuilder, F: FnMut(EntityWorldMut)> {
    builder: B,
    f: F,
}

pub struct ForEachAdded<B: ExchangeBuilder, F: FnMut(EntityWorldMut)> {
    builder: B,
    f: F,
}

impl<B: ExchangeBuilder, F: FnMut(EntityWorldMut)> ExchangeBuilder for ForEachRemoved<B, F> {
    fn root(&self, world: &World) -> Option<Entity> {
        self.builder.root(world)
    }

    fn invoke_for_each_removed(&mut self, mut entity: EntityWorldMut) {
        entity.reborrow_scope(|e| self.builder.invoke_for_each_removed(e));
        entity.reborrow_scope(|e| (self.f)(e));
    }

    fn invoke_for_each_added(&mut self, entity: EntityWorldMut) {
        self.builder.invoke_for_each_added(entity);
    }
}

impl<B: ExchangeBuilder, F: FnMut(EntityWorldMut)> ExchangeBuilder for ForEachAdded<B, F> {
    fn root(&self, world: &World) -> Option<Entity> {
        self.builder.root(world)
    }

    fn invoke_for_each_removed(&mut self, entity: EntityWorldMut) {
        self.builder.invoke_for_each_removed(entity);
    }

    fn invoke_for_each_added(&mut self, mut entity: EntityWorldMut) {
        entity.reborrow_scope(|e| self.builder.invoke_for_each_added(e));
        entity.reborrow_scope(|e| (self.f)(e));
    }
}

impl ExchangeChildrenExt for World {
    fn exchange_children(
        &mut self,
        entity: Entity,
        incoming: impl IntoIterator<Item = Entity>,
        mut arguments: impl ExchangeBuilder,
    ) {
        let root = arguments.root(self);
        let mut removed = self
            .get::<Children>(entity)
            .map(|x| x.collection().clone())
            .unwrap_or_default();
        let incoming = incoming
            .into_iter()
            .filter(|e| self.get_entity(*e).inspect_err(|e| {
                warn!("In exchange_children, entity {e} missing!");
            }).is_ok())
            .collect::<Vec<_>>();
        let mut added = incoming.clone();
        for v in &removed {
            if let Some(pos) = added.iter().position(|x| x == v) {
                added.remove(pos);
            }
        }
        for item in &incoming {
            if let Some(pos) = removed.iter().position(|x| x == item) {
                removed.remove(pos);
            }
        }
        if let Some(root) = root {
            for e in &removed {
                if let Ok(mut e) = self.get_entity_mut(*e) {
                    e.insert(ChildOf(root));
                }
            }
        } else {
            for e in &removed {
                if let Ok(mut e) = self.get_entity_mut(*e) {
                    e.remove::<ChildOf>();
                }
            }
        }
        for e in &added {
            if let Ok(mut e) = self.get_entity_mut(*e) {
                e.insert(ChildOf(entity));
            }
        }
        if let Some(mut children) = self.get_mut::<Children>(entity) {
            sort_by_order(children.collection_mut_risky(), &incoming)
        }
        for e in removed {
            if let Ok(e) = self.get_entity_mut(e) {
                arguments.invoke_for_each_removed(e);
            }
        }
        for e in added {
            if let Ok(e) = self.get_entity_mut(e) {
                arguments.invoke_for_each_added(e);
            }
        }
    }
}

fn sort_by_order(arr: &mut [Entity], reference: &[Entity]) {
    arr.sort_by_cached_key(|a| reference.iter().position(|b| a == b));
}

impl ExchangeChildrenExt for App {
    fn exchange_children(
        &mut self,
        entity: Entity,
        incoming: impl IntoIterator<Item = Entity>,
        arguments: impl ExchangeBuilder,
    ) {
        self.world_mut()
            .exchange_children(entity, incoming, arguments);
    }
}

impl ExchangeChildrenCommandExt for Commands<'_, '_> {
    fn exchange_children(
        &mut self,
        entity: Entity,
        incoming: impl IntoIterator<Item = Entity>,
        arguments: impl ExchangeBuilder + Send + Sync + 'static,
    ) {
        let incoming = incoming.into_iter().collect::<Vec<_>>();
        self.queue(move |world: &mut World| {
            world.exchange_children(entity, incoming, arguments);
        });
    }
}
