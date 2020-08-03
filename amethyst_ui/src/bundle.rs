use crate::{
    renderer::UiGlyphsResource, selection::*, selection_order_cache::SelectionOrderCache,
    sorted::SortedWidgets, systems, FontAsset, UiEvent,
};
use amethyst_assets::AssetStorage;
use amethyst_core::{
    dispatcher::{DispatcherBuilder, Stage, SystemBundle},
    ecs::prelude::*,
    shrev::EventChannel,
};
use amethyst_error::Error;
use amethyst_input::{BindingTypes, InputEvent, InputHandler};
use std::marker::PhantomData;

/// Registers all resources and system necessary for an in-game UI.
/// * `T` represents the `BindingTypes` used by `InputHandler`.
/// * `G` represents the type used to distinguish between selection groups.
#[derive(Default, Debug)]
pub struct UiBundle<T, G = ()>
where
    T: BindingTypes,
    G: Send + Sync + PartialEq + 'static,
{
    _phantom: PhantomData<(T, G)>,
}

impl<T, G> UiBundle<T, G>
where
    T: BindingTypes,
    G: Send + Sync + PartialEq + 'static,
{
    /// Creates a new `UiBundle`.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T, G> SystemBundle for UiBundle<T, G>
where
    T: BindingTypes,
    G: Send + Sync + PartialEq + 'static,
{
    fn build(
        self,
        _world: &mut World,
        resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), Error> {
        resources.insert(AssetStorage::<FontAsset>::new());
        resources.insert(UiGlyphsResource::default());
        resources.insert(EventChannel::<UiEvent>::new());
        resources.insert(SortedWidgets::new());
        resources.insert(SelectedEntities::default());
        resources.insert(SelectionOrderCache::default());

        // TODO: Remove; should be handled by `amethyst_input`
        resources.insert(InputHandler::<T>::new());
        resources.insert(EventChannel::<InputEvent<T>>::new());

        builder.add_system(Stage::Logic, systems::build_font_asset_processor_system);
        builder.add_system(Stage::Logic, systems::build_ui_transform_system);
        builder.add_system(Stage::Logic, systems::build_ui_sorting_system);
        builder.add_system(Stage::Logic, systems::build_ui_mouse_system::<T>);
        builder.add_system(Stage::Logic, systems::build_drag_widget_system::<T>);
        builder.add_system(Stage::Logic, systems::build_text_editing_input_system);
        builder.add_system(
            Stage::Logic,
            systems::build_ui_button_action_retrigger_system,
        );
        builder.add_system(Stage::Logic, systems::build_ui_button_system);
        builder.add_system(
            Stage::Logic,
            systems::build_selection_order_cache_system::<G>,
        );
        builder.add_system(Stage::Logic, systems::build_mouse_selection_system::<T, G>);

        // Removed; requires `Output` resource
        // builder.add_system(Stage::Logic, systems::build_ui_sound_system);

        Ok(())
    }
}
