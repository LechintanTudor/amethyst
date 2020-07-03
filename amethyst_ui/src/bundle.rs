use crate::systems;
use amethyst_core::{
    dispatcher::{DispatcherBuilder, Stage, SystemBundle},
    ecs::prelude::*,
};
use amethyst_error::Error;
use amethyst_input::BindingTypes;
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
        builder.add_system(Stage::Logic, systems::build_ui_mouse_system::<T>);
        builder.add_system(Stage::Logic, systems::build_font_asset_processor_system);

        todo!("loader");
        todo!("transform");
        // todo!("mouse");
        // todo!("processor -> font asset");
        todo!("cache selection order");
        todo!("selection mouse");
        todo!("selection keyboard");
        todo!("text editing mouse");
        todo!("text editing input");
        todo!("resize");
        todo!("button");
        todo!("drag widget");
        todo!("action retrigger");
        todo!("sound");
        todo!("sound retrigger");
        todo!("blink");
        todo!()
    }
}