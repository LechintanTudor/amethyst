use crate::{systems, EventReceiver, EventRetrigger, UiButtonAction, UiEvent, UiEventType};
use amethyst_core::ecs::prelude::*;

#[derive(Clone, Default, Debug)]
pub struct UiButtonActionRetrigger {
    pub on_click_start: Vec<UiButtonAction>,
    pub on_click_stop: Vec<UiButtonAction>,
    pub on_hover_start: Vec<UiButtonAction>,
    pub on_hover_stop: Vec<UiButtonAction>,
}

impl EventRetrigger for UiButtonActionRetrigger {
    type In = UiEvent;
    type Out = UiButtonAction;

    fn apply<R>(&self, event: &Self::In, receiver: &mut R)
    where
        R: EventReceiver<Self::Out>,
    {
        match event.event_type {
            UiEventType::ClickStart => receiver.receive(&self.on_click_start),
            UiEventType::ClickStop => receiver.receive(&self.on_click_stop),
            UiEventType::HoverStart => receiver.receive(&self.on_hover_start),
            UiEventType::HoverStop => receiver.receive(&self.on_hover_stop),
            _ => (),
        }
    }
}

pub fn build_ui_button_action_retrigger_system(
    world: &mut World,
    resources: &mut Resources,
) -> Box<dyn Schedulable> {
    systems::build_event_retrigger_system::<UiButtonActionRetrigger>(world, resources)
}
