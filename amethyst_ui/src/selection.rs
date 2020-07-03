use amethyst_core::{
    ecs::prelude::*,
    shrev::EventChannel,
};
use serde::{Deserialize, Serialize};
use winit::Event;

#[derive(Debug, Serialize, Deserialize)]
pub struct Selectable<G> {
    pub order: u32,
    pub multi_select_group: Option<G>,
    pub auto_multi_select: bool,
    pub consumes_inputs: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Selected;

pub fn build_keyboard_selection_system<G>(
    _world: &mut World, resources: &mut Resources) -> Box<dyn Schedulable>
where G: Send + Sync + 'static + PartialEq
{
    let mut winit_event_reader_id = resources
        .get_mut::<EventChannel<Event>>()
        .expect("`EventChannel<Event>` was not found in resources")
        .register_reader();

    todo!()
}