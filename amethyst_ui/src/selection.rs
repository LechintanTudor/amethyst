use crate::{UiEvent, UiEventType};
use amethyst_core::{
    ecs::prelude::*,
    shrev::EventChannel,
};
use amethyst_input::{BindingTypes, InputHandler};
use std::collections::HashSet;
use winit::VirtualKeyCode;

#[derive(Clone, Default, Debug)]
pub struct SelectedEntities {
    entities: HashSet<Entity>,
    last_entity: Option<Entity>,
}

impl SelectedEntities {
    pub fn clear(&mut self) {
        self.entities.clear();
        self.last_entity = None;
    }

    pub fn insert(&mut self, entity: Entity) {
        self.entities.insert(entity);
        self.last_entity = Some(entity);
    }

    pub fn remove(&mut self, entity: Entity) {
        self.entities.remove(&entity);

        if matches!(self.last_entity, Some(entity)) {
            self.last_entity = self.entities.iter().next().cloned();
        }
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }

    pub fn entities(&self) -> &HashSet<Entity> {
        &self.entities
    }

    pub fn last_entity(&self) -> Option<Entity> {
        self.last_entity
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Selectable<G>
where
    G: Send + Sync + PartialEq + 'static,
{
    pub order: u32,
    pub multi_select_group: Option<G>,
    pub auto_multi_select: bool,
    pub consumes_inputs: bool,
}

pub fn build_mouse_selection_system<T, G>(_world: &mut World, resources: &mut Resources) -> Box<dyn Schedulable>
where
    T: BindingTypes,
    G: Send + Sync + PartialEq + 'static,
{
    let mut ui_event_reader = resources
        .get_mut_or_default::<EventChannel<UiEvent>>()
        .unwrap()
        .register_reader();

    let mut emitted_ui_events = Vec::<UiEvent>::new();

    SystemBuilder::<()>::new("MouseSelectionSystem")
        .read_resource::<InputHandler<T>>()
        .write_resource::<EventChannel<UiEvent>>()
        .write_resource::<SelectedEntities>()
        .read_component::<Selectable<G>>()
        .build(move |_, world, resources, _| {
            let (input, ui_events, selected) = resources;
            let ctrl = input.key_is_down(VirtualKeyCode::LControl)
                | input.key_is_down(VirtualKeyCode::RControl);

            for event in ui_events.read(&mut ui_event_reader) {
                if matches!(event.event_type, UiEventType::ClickStart) {
                    let entity = event.target;

                    let selectable = match world.get_component::<Selectable<G>>(entity) {
                        Some(selectable) => selectable,
                        None => {
                            emitted_ui_events.extend(
                                selected.entities.drain()
                                    .map(|e| UiEvent::new(UiEventType::Blur, e))
                            );
                            continue;
                        }
                    };

                    let same_select_group = {
                        if let Some(last_entity) = selected.last_entity() {
                            if let Some(last_selectable) = world.get_component::<Selectable<G>>(last_entity) {
                                last_selectable.multi_select_group == selectable.multi_select_group
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    };

                    if same_select_group && (ctrl || selectable.auto_multi_select) {
                        selected.insert(entity);
                        emitted_ui_events.push(UiEvent::new(UiEventType::Focus, entity));
                    } else {
                        for &entity in selected.entities() {
                            emitted_ui_events.push(UiEvent::new(UiEventType::Blur, entity));
                        }

                        selected.clear();
                        selected.insert(entity);

                        emitted_ui_events.push(UiEvent::new(UiEventType::Focus, entity));
                    }
                }
            }

            ui_events.iter_write(emitted_ui_events.drain(..));
        })
}