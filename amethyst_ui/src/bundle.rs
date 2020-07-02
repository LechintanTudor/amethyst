use amethyst_core::{
    dispatcher::{DispatcherBuilder, SystemBundle},
    ecs::prelude::*,
};
use amethyst_error::Error;

#[derive(Debug)]
pub struct UiBundle;

impl SystemBundle for UiBundle {
    fn build(self, world: &mut World, resources: &mut Resources, builder: &mut DispatcherBuilder<'_>) -> Result<(), Error> {
        todo!("loader");
        todo!("transform");
        todo!("mouse");
        todo!("processor -> font asset");
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