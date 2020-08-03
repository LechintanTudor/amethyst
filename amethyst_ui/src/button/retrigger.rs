use crate::{systems, EventReceiver, EventRetrigger, UiButtonAction, UiEvent, UiEventType};
use amethyst_core::ecs::prelude::*;

/// Used to trigger events on a `UiButton` when a user
/// interacion happens
#[derive(Clone, Default, Debug)]
pub struct UiButtonActionRetrigger {
    /// The `UiButtonAction`s to be triggered when the user begins a click on the `UiButton`
    pub on_click_start: Vec<UiButtonAction>,
    /// The `UiButtonAction`s to be triggered when the user ends a click on the `UiButton`
    pub on_click_stop: Vec<UiButtonAction>,
    /// The `UiButtonAction`s to be triggered when the user starts hovering over the `UiButton`
    pub on_hover_start: Vec<UiButtonAction>,
    /// The `UiButtonAction`s to be triggered when the user stops hovering over the `UiButton`
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

/// Builds a system that triggers `UiButtonAction`s based on user interaction.
pub fn build_ui_button_action_retrigger_system(
    world: &mut World,
    resources: &mut Resources,
) -> Box<dyn Schedulable> {
    systems::build_event_retrigger_system::<UiButtonActionRetrigger>(world, resources)
}
