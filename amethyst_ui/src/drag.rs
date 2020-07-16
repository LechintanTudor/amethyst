use crate::{
    Parent, ScaleMode, UiEvent, UiEventType, UiTransform,
    event,
    sorted::SortedWidgets,
    transform,
    utils,
};
use amethyst_core::{
    Hidden, HiddenPropagate,
    ecs::prelude::*,
    math::Vector2,
    shrev::{EventChannel, ReaderId},
};
use amethyst_input::{BindingTypes, InputHandler};
use amethyst_window::ScreenDimensions;
use serde::{Deserialize, Serialize};
use std::{
    any,
    collections::HashSet,
};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Draggable;

pub fn build_drag_widget_system<T>(world: &mut World, resources: &mut Resources) -> Box<dyn Schedulable>
where T: BindingTypes
{
    let mut ui_reader_id = resources
        .get_mut::<EventChannel<UiEvent>>()
        .expect("`EventChannel<UiEvent>` was not found in resources")
        .register_reader();

    let mut start_mouse_position = Vector2::<f32>::new(0.0, 0.0);
    let mut last_mouse_position = Vector2::<f32>::new(0.0, 0.0);
    let mut dragged_targets = HashSet::<Entity>::new();
    let mut drag_stop_targets = HashSet::<Entity>::new();

    SystemBuilder::<()>::new("DragWidgetSystem")
        .read_resource::<InputHandler<T>>()
        .read_resource::<ScreenDimensions>()
        .read_resource::<SortedWidgets>()
        .write_resource::<EventChannel<UiEvent>>()
        .read_component::<Draggable>()
        .read_component::<Hidden>()
        .read_component::<HiddenPropagate>()
        .read_component::<Parent>()
        .write_component::<UiTransform>()
        .build(move |_, world, resources, _| {
            let (input, screen_dimensions, sorted_widgets, ui_events) = resources;
            let mouse_position = input.mouse_position().unwrap_or((0.0, 0.0));
            let mouse_position = utils::world_position(mouse_position, &screen_dimensions);
            let mouse_position = Vector2::new(mouse_position.0, mouse_position.1);

            drag_stop_targets.clear();

            for event in ui_events.read(&mut ui_reader_id) {
                match event.event_type {
                    UiEventType::ClickStart => {
                        start_mouse_position = mouse_position;

                        if world.has_component::<Draggable>(event.target) {
                            dragged_targets.insert(event.target);
                        }
                    }
                    UiEventType::ClickStop => {
                        if dragged_targets.contains(&event.target) {
                            drag_stop_targets.insert(event.target);
                        }
                    }
                    _ => (),
                }
            }

            for &entity in dragged_targets.iter() {
                if world.has_component::<Hidden>(entity) ||
                    world.has_component::<HiddenPropagate>(entity)
                {
                    drag_stop_targets.insert(entity);
                }

                ui_events.single_write(UiEvent::new(
                    UiEventType::Dragging {
                        offset_from_mouse: mouse_position - start_mouse_position,
                        new_position: mouse_position,
                    },
                    entity,
                ));

                let change = mouse_position - last_mouse_position;
                let (parent_width, parent_height) =
                    transform::get_parent_pixel_size(entity, world, &screen_dimensions);

                if let Some(mut transform) = world.get_component_mut::<UiTransform>(entity) {
                    let (scale_x, scale_y) = match transform.scale_mode {
                        ScaleMode::Pixel => (1.0, 1.0),
                        ScaleMode::Percent => (parent_width, parent_height),
                    };

                    transform.local_x += change.x / scale_x;
                    transform.local_y += change.y / scale_y;
                }
            }

            last_mouse_position = mouse_position;

            for &entity in drag_stop_targets.iter() {
                if let Some(transform) = world.get_component::<UiTransform>(entity) {
                    ui_events.single_write(UiEvent::new(
                        UiEventType::Dropped {
                            dropped_on: event::get_targeted_below(
                                (mouse_position.x, mouse_position.y),
                                transform.global_z,
                                &sorted_widgets,
                                world,
                            )
                        },
                        entity,
                    ));
                }

                dragged_targets.remove(&entity);
            }
        })
}