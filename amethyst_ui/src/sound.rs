use crate::{EventReceiver, EventRetrigger, UiEvent, UiEventType};
use amethyst_assets::AssetStorage;
use amethyst_audio::{
    Source, SourceHandle,
    output::Output,
};
use amethyst_core::{
    ecs::prelude::*,
    shrev::EventChannel,
};

#[derive(Clone, Debug)]
pub struct UiPlaySoundAction(pub SourceHandle);

#[derive(Clone, Debug)]
pub struct UiSoundRetrigger {
    pub on_click_start: Option<UiPlaySoundAction>,
    pub on_click_stop: Option<UiPlaySoundAction>,
    pub on_hover_start: Option<UiPlaySoundAction>,
    pub on_hover_stop: Option<UiPlaySoundAction>,
}

impl EventRetrigger for UiSoundRetrigger {
    type In = UiEvent;
    type Out = UiPlaySoundAction;

    fn apply<R>(&self, event: &Self::In, receiver: &mut R)
    where
        R: EventReceiver<Self::Out>
    {
        let event_to_trigger = match event.event_type {
            UiEventType::ClickStart => &self.on_click_start,
            UiEventType::ClickStop => &self.on_click_stop,
            UiEventType::HoverStart => &self.on_hover_start,
            UiEventType::HoverStop => &self.on_hover_stop,
            _ => return,
        };

        if let Some(event) = event_to_trigger {
            receiver.receive_one(event);
        }
    }
}

pub fn build_ui_sound_system(_: &mut World, resources: &mut Resources) -> Box<dyn Schedulable> {
    let mut play_sound_action_reader = resources
        .get_mut_or_default::<EventChannel<UiPlaySoundAction>>()
        .unwrap()
        .register_reader();

    SystemBuilder::<()>::new("UiSoundSystem")
        .read_resource::<Option<Output>>()
        .read_resource::<AssetStorage<Source>>()
        .write_resource::<EventChannel<UiPlaySoundAction>>()
        .build(move |_, _, resources, _| {
            let (output, source_storage, play_sound_actions) = resources;

            for action in play_sound_actions.read(&mut play_sound_action_reader) {
                if let Some(output) = output.as_ref() {
                    if let Some(sound) = source_storage.get(&action.0) {
                        output.play_once(sound, 1.0);
                    }
                }
            }
        })
}