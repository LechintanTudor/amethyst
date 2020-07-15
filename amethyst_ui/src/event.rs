use crate::{
    UiTransform,
    sorted::SortedWidgets,
    utils,
};
use amethyst_core::{
    Hidden, HiddenPropagate,
    ecs::prelude::*,
    math::Vector2,
    shrev::EventChannel,
};
use amethyst_input::{BindingTypes, InputHandler};
use amethyst_window::ScreenDimensions;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use winit::MouseButton;

pub trait TargetedEvent {
    fn target(&self) -> Entity;
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiEventType {
    Click,
    ClickStart,
    ClickStop,
    HoverStart,
    HoverStop,
    Dragging {
        offset_from_mouse: Vector2<f32>,
        new_position: Vector2<f32>,
    },
    Dropped {
        dropped_on: Option<Entity>,
    },
    ValueChange,
    ValueCommit,
    Focus,
    Blur,
}

#[derive(Clone, Debug)]
pub struct UiEvent {
    pub event_type: UiEventType,
    pub target: Entity,
}

impl TargetedEvent for UiEvent {
    fn target(&self) -> Entity {
        self.target
    }
}

impl UiEvent {
    pub fn new(event_type: UiEventType, target: Entity) -> Self {
        Self { event_type, target }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interactable;

pub fn build_ui_mouse_system2<T>(_: &mut World, _: &mut Resources) -> Box<dyn Schedulable>
where T: BindingTypes
{
    SystemBuilder::<()>::new("UiMouseSystem")
        .read_resource::<InputHandler<T>>()
        .read_resource::<ScreenDimensions>()
        .read_resource::<SortedWidgets>()
        .write_resource::<EventChannel<UiEvent>>()
        .build(move |_, world, resources, queries| {

        })
}

pub fn build_ui_mouse_system<T>(_world: &mut World, _resources: &mut Resources) -> Box<dyn Schedulable>
where T: BindingTypes
{
    let mut mouse_was_down = false;
    let mut click_started_on = HashSet::<Entity>::new();
    let mut last_targets = HashSet::<Entity>::new();

    SystemBuilder::<()>::new("UiMouseSystem")
        .read_resource::<InputHandler<T>>()
        .read_resource::<ScreenDimensions>()
        .read_resource::<SortedWidgets>()
        .write_resource::<EventChannel<UiEvent>>()
        .read_component::<UiTransform>()
        .build(move |_, world, resources, _| {
            let (input, screen_dimensions, sorted_widgets, events) = resources;

            let mouse_down = input.mouse_button_is_down(MouseButton::Left);
            let click_started = mouse_down && !mouse_was_down;
            let click_stopped = !mouse_down && mouse_was_down;

            if let Some(mouse_position) = input.mouse_position() {
                let mouse_position = utils::world_position(mouse_position, &screen_dimensions);

                let targets = get_targeted_entities(mouse_position, &sorted_widgets, world);

                for target in targets.difference(&last_targets) {
                    events.single_write(UiEvent::new(UiEventType::HoverStart, *target));
                }

                for target in last_targets.difference(&targets) {
                    events.single_write(UiEvent::new(UiEventType::HoverStop, *target));
                }

                if click_started {
                    click_started_on = targets.clone();

                    for target in targets.iter() {
                        events.single_write(UiEvent::new(UiEventType::ClickStart, *target));
                    }
                } else if click_stopped {
                    for target in click_started_on.intersection(&targets) {
                        events.single_write(UiEvent::new(UiEventType::Click, *target));
                    }
                }

                last_targets = targets;
            }

            if click_stopped {
                for target in click_started_on.drain() {
                    events.single_write(UiEvent::new(UiEventType::ClickStop, target));
                }
            }

            mouse_was_down = mouse_down;
        })
}

pub fn get_targeted_entities<E>(
    (mouse_x, mouse_y): (f32, f32),
    sorted_widgets: &SortedWidgets,
    world: &E
) -> HashSet<Entity>
where E: EntityStore
{
    let mut entities = HashSet::<Entity>::new();

    for &(entity, _) in sorted_widgets.widgets().iter().rev() {
        if let Some(transform) = world.get_component::<UiTransform>(entity) {
            if transform.position_inside(mouse_x, mouse_y) {
                if transform.opaque {
                    entities.insert(entity);
                    break;
                } else if transform.transparent_target {
                    entities.insert(entity);
                }
            }
        }
    }

    entities
}