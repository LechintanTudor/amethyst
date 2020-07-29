use crate::Selectable;
use amethyst_core::ecs::prelude::*;
use std::cmp::Ordering;

#[derive(Clone, Default, Debug)]
pub struct SelectionOrderCache {
    pub cache: Vec<(u32, Entity)>,
}

pub fn build_selection_order_cache_system<G>(_: &mut World, _: &mut Resources) -> Box<dyn Schedulable>
where
    G: Send + Sync + PartialEq + 'static,
{
    SystemBuilder::<()>::new("SelectionOrderCacheSystem")
        .write_resource::<SelectionOrderCache>()
        .with_query(
            TryRead::<Selectable<G>>::query()
                .filter(changed::<Selectable<G>>())
        )
        .with_query(Read::<Selectable<G>>::query())
        .build(|_, world, selection_cache, queries| {
            // Cache was invalidated
            if queries.0.iter(world).next().is_some() {
                selection_cache.cache.clear();
                selection_cache.cache.extend(
                    queries.1.iter_entities(world)
                        .map(|(e, s)| (s.order, e))
                );
                selection_cache.cache.sort_by(|(o1, e1), (o2, e2)| {
                    match o1.cmp(o2) {
                        Ordering::Equal => e1.index().cmp(&e2.index()),
                        ordering => ordering,
                    }
                });
            }
        })
}