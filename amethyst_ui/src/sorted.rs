use crate::UiTransform;
use amethyst_core::{ecs::prelude::*, Hidden, HiddenPropagate};

// Maintains a cache of sorted widgets and their Z position.
#[derive(Clone, Debug)]
pub struct SortedWidgets {
    widgets: Vec<(Entity, f32)>,
}

impl SortedWidgets {
    /// Creates a new `SortedWidgets` instance.
    pub fn new() -> Self {
        Self {
            widgets: Vec::new(),
        }
    }

    /// Returns a slice of tuples where the first element is the entity
    /// which represents the UI element and the second is its Z position.
    pub fn widgets(&self) -> &[(Entity, f32)] {
        &self.widgets
    }
}

pub(crate) fn build_ui_sorting_system(_: &mut World, _: &mut Resources) -> Box<dyn Schedulable> {
    SystemBuilder::<()>::new("UiSortingSystem")
        .with_query(
            Read::<UiTransform>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>()),
        )
        .write_resource::<SortedWidgets>()
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
