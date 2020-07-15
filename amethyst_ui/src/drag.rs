use crate::{
    UiEvent, UiEventType,
    utils,
};
use amethyst_core::{
    ecs::prelude::*,
    math::Vector2,
    shrev::{EventChannel, ReaderId},
};
use amethyst_input::{BindingTypes, InputHandler};
use amethyst_window::ScreenDimensions;
use serde::{Deserialize, Serialize};
use std::any;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Draggable;

pub fn build_drag_widget_system<T>(world: &mut World, resources: &mut Resources) -> Box<dyn Schedulable>
where T: BindingTypes
{
    let mut ui_reader_id = resources
        .get_mut::<EventChannel<UiEvent>>()
        .expect("`EventChannel<UiEvent>` was not found in resources")
        .register_reader();

    SystemBuilder::<()>::new("DragWidgetSystem")
        .read_resource::<InputHandler<T>>()
        .write_resource::<EventChannel<UiEvent>>()
        .read_resource::<ScreenDimensions>()
        .build(move |_, _, resources, _| {
            let (input, ui_events, screen_dimensions) = resources;

            let mouse_position = input.mouse_position().unwrap_or((0.0, 0.0));
            let mouse_position = utils::world_position(mouse_position, &screen_dimensions);

            for ui_event in ui_events.read(&mut ui_reader_id) {
                println!("{:?}", ui_event);
            }
        })
}