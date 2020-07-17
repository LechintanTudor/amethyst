use crate::{
    FontAsset, UiEvent,
    glyphs::UiGlyphsResource,
    selection::Selected,
    sorted::SortedWidgets,
    systems,
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

#[derive(Default, Debug)]
pub struct UiBundle<T>
where T: BindingTypes
{
    _phantom: PhantomData<T>
}

impl<T> UiBundle<T>
where T: BindingTypes
{
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> SystemBundle for UiBundle<T>
where T: BindingTypes
{
    fn build(self, world: &mut World, resources: &mut Resources, builder: &mut DispatcherBuilder<'_>) -> Result<(), Error> {
        resources.insert(AssetStorage::<FontAsset>::new());
        resources.insert(UiGlyphsResource::default());
        resources.insert(EventChannel::<UiEvent>::new());
        resources.insert(SortedWidgets::new());
        resources.insert(Selected::default());

        // TODO: Remove; should be handled by `amethyst_input`
        resources.insert(InputHandler::<T>::new());
        resources.insert(EventChannel::<InputEvent<T>>::new());

        builder.add_system(Stage::Logic, systems::build_font_asset_processor_system);
        builder.add_system(Stage::Logic, systems::build_ui_transform_system);
        builder.add_system(Stage::Logic, systems::build_ui_sorting_system);
        builder.add_system(Stage::Logic, systems::build_ui_mouse_system::<T>);
        builder.add_system(Stage::Logic, systems::build_drag_widget_system::<T>);
        builder.add_system(Stage::Logic, systems::build_text_editing_input_system);

        Ok(())
    }
}