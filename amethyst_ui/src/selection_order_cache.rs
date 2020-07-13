use crate::Selected;
use amethyst_core::ecs::prelude::*;
use hibitset::BitSet;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

#[derive(Clone, Default, Debug)]
pub struct CachedSelectionOrder {
    pub cached: BitSet,
    pub cache: Vec<(u32, Entity)>,
}

impl CachedSelectionOrder {
    pub fn get_highest_order_selected_index<I, S>(&self, selected_iter: I) -> Option<usize>
    where
        I: Iterator<Item = S>,
        S: AsRef<Selected>,
    {
        todo!()
    }

    pub fn index_of(&self, entity: Entity) -> Option<usize> {
        self.cache
            .iter()
            .enumerate()
            .find(|(_, (_, e))| *e == entity)
            .map(|(i, (_, _))| i)
    }
}

pub fn build_cache_selection_order_system<G>(
    _world: &mut World, _resources: &mut Resources) -> Box<dyn Schedulable>
where G: Send + Sync + 'static + PartialEq
{
    SystemBuilder::<()>::new("CacheSelectionOrderSystem")
        .write_resource::<CachedSelectionOrder>()
        .build(|_, world, resource, _| {

        });

    todo!()
}