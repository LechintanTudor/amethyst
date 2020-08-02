use crate::UiTransform;
use amethyst_core::{ecs::prelude::*, Hidden, HiddenPropagate};

// Sorts visible widgets from farthest to closest
#[derive(Clone, Debug)]
pub struct SortedWidgets {
    widgets: Vec<(Entity, f32)>,
}

impl SortedWidgets {
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }

    pub fn widgets(&self) -> &[(Entity, f32)] {
        &self.widgets
    }
}

pub fn build_ui_sorting_system(_: &mut World, _: &mut Resources) -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("UiSortingSystem")
        .write_resource::<SortedWidgets>()
        .with_query(
            Read::<UiTransform>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>()),
        )
        .build(|_, world, sorted, query| {
            sorted.widgets.clear();
            sorted
                .widgets
                .extend(query.iter_entities(world).map(|(e, t)| (e, t.global_z)));
            sorted
                .widgets
                .sort_by(|(_, z1), (_, z2)| f32::partial_cmp(z1, z2).expect("Unexpected NaN"));
        })
}
