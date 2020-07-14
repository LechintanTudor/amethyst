use crate::{UiEvent, UiEventType};
use amethyst_core::{
    ecs::prelude::*,
    shrev::{EventChannel, ReaderId},
};
use amethyst_input::{BindingTypes, InputHandler};
use amethyst_window::ScreenDimensions;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Draggable;

pub fn build_draw_widget_system<B>(world: &mut World, resources: &mut Resources) -> Box<dyn Schedulable>
where B: BindingTypes
{
    SystemBuilder::<()>::new("DragWidgetSystem")
        .read_resource::<InputHandler<B>>()
        .write_resource::<EventChannel<UiEvent>>()
        .read_resource::<ScreenDimensions>()
        .build(|_, _, _, _| {

        });

    todo!()
}