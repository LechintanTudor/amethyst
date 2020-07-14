use crate::UiTransform;
use amethyst_core::{
    Hidden, HiddenPropagate,
    ecs::prelude::*,
};

// Sorts visible widgets from farthest to closest
#[derive(Clone, Debug)]
pub struct SortedWidgets {
    entities: Vec<(Entity, f32)>,
}

impl SortedWidgets {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    pub fn entities(&self) -> &[(Entity, f32)] {
        &self.entities
    }
}

pub fn build_ui_sorting_system(_: &mut World, _: &mut Resources) -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("UiSortingSystem")
        .write_resource::<SortedWidgets>()
        .with_query(
            Read::<UiTransform>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>())
        )
        .build(|_, world, sorted_widgets, query| {
            sorted_widgets.entities.clear();
            sorted_widgets.entities.extend(
                query
                    .iter_entities(world)
                    .map(|(e, t)| (e, t.global_z))
            );
            sorted_widgets
                .entities
                .sort_by(|(_, z1), (_, z2)| f32::partial_cmp(z1, z2).expect("Unexpected NaN"));
        })
}