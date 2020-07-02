use crate::UiTransform;
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

pub fn build_ui_mouse_system<T>() -> Box<dyn Schedulable>
where T: BindingTypes
{
    let mut mouse_was_down = false;
    let mut click_started_on = HashSet::<Entity>::new();
    let mut last_targets = HashSet::<Entity>::new();

    SystemBuilder::<()>::new("UiMouseSystem")
        .read_resource::<InputHandler<T>>()
        .read_resource::<ScreenDimensions>()
        .write_resource::<EventChannel<UiEvent>>()
        .with_query(Read::<UiTransform>::query()
            .filter(!component::<Hidden>() & !component::<HiddenPropagate>())
        )
        .build(move |_, world, resources, query| {
            let (input, screen_dimensions, events) = resources;

            let mouse_down = input.mouse_button_is_down(MouseButton::Left);
            let click_started = mouse_down && !mouse_was_down;
            let click_stopped = !mouse_down && mouse_was_down;

            if let Some((mouse_x, mouse_y)) = input.mouse_position() {
                let x = mouse_x as f32;
                // Invert Y to match Amethyst's coord system
                let y = screen_dimensions.height() - mouse_y as f32;

                let targets = get_targeted_entities((x, y), query.iter_entities(world));

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

pub fn get_targeted_entities<I, T>((mouse_x, mouse_y): (f32, f32), transform_iter: I) -> HashSet<Entity>
where
    I: Iterator<Item = (Entity, T)>,
    T: AsRef<UiTransform>,
{
    // Get hovered transforms
    let mut transforms = transform_iter
        .filter(|(_, t)| {
            let t = t.as_ref();

            (t.opaque || t.transparent_target) && t.position_inside(mouse_x, mouse_y)
        })
        .collect::<Vec<_>>();

    // Sort transforms from closest to farthest
    transforms.sort_by(|(_, t1), (_, t2)| {
        (t2.as_ref().global_z).partial_cmp(&t1.as_ref().global_z).expect("Unexpected NaN")
    });

    // Discard transforms after first opaque
    let first_opaque = transforms.iter().position(|(_, t)| t.as_ref().opaque);
    if let Some(i) = first_opaque {
        transforms.truncate(i + 1);
    }

    transforms.into_iter().map(|(e, _)| e).collect()
}