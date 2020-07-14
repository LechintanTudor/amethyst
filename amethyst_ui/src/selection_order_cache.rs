use crate::Selected;
use amethyst_core::ecs::prelude::*;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

pub fn build_cache_selection_order_system<G>(
    _world: &mut World, _resources: &mut Resources) -> Box<dyn Schedulable>
where G: Send + Sync + 'static + PartialEq
{
    SystemBuilder::<()>::new("CacheSelectionOrderSystem")
        .build(|_, world, resources, _| {

        });

    todo!()
}