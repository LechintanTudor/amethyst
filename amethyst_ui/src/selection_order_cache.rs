use crate::Selectable;
use amethyst_core::ecs::prelude::*;
use std::cmp::Ordering;

/// Maintains a cache of selectable entities and their tab order.
#[derive(Clone, Default, Debug)]
pub struct SelectionOrderCache {
    cache: Vec<(Entity, u32)>,
}

impl SelectionOrderCache {
    /// Returns a slice of tuples where the first element is the entity
    /// which represents the UI element and the second is the tab order
    pub fn entitites(&self) -> &[(Entity, u32)] {
        &self.cache
    }
}

pub(crate) fn build_selection_order_cache_system<G>(
    _world: &mut World,
    _resources: &mut Resources,
) -> Box<dyn Schedulable>
where
    G: Send + Sync + PartialEq + 'static,
{
    SystemBuilder::<()>::new("SelectionOrderCacheSystem")
        .with_query(TryRead::<Selectable<G>>::query().filter(changed::<Selectable<G>>()))
        .with_query(Read::<Selectable<G>>::query())
        .write_resource::<SelectionOrderCache>()
        .build(|_, world, selection_cache, queries| {
            selection_cache.cache.clear();
            selection_cache
                .cache
                .extend(queries.1.iter_entities(world).map(|(e, s)| (e, s.order)));
            selection_cache
                .cache
                .sort_by(|(e1, o1), (e2, o2)| match o1.cmp(o2) {
                    Ordering::Equal => e1.index().cmp(&e2.index()),
                    ordering => ordering,
                });
        })
}
