use crate::{sorted::SortedWidgets, UiTransform};
use amethyst_core::{ecs::prelude::*, math::Vector2, shrev::EventChannel};
use amethyst_input::{BindingTypes, InputHandler};
use amethyst_window::ScreenDimensions;
use std::collections::HashSet;
use winit::MouseButton;

/// Trait implemented by events which target a specific `Entity`
pub trait TargetedEvent {
    fn target(&self) -> Entity;
}

/// The type of `UiEvent`
#[derive(Clone, Debug, PartialEq)]
pub enum UiEventType {
    /// Click started and stopped on the same UI element
    Click,
    /// Click started on a UI element
    ClickStart,
    /// Click stopped on a UI element
    ClickStop,
    /// Cursor started hovering above a UI element
    HoverStart,
    /// Cursor stopped hovering above a UI element
    HoverStop,
    /// Sent repeatedly when dragging a UI element
    Dragging {
        /// Position of the cursor relative to the center of the `UiTransform`
        /// when drag started
        offset_from_mouse: Vector2<f32>,
        /// Absolute cursor position at the current time
        new_position: Vector2<f32>,
    },
    /// Stopped dragging a UI element
    Dropped {
        /// The entity with a `UiTransform` on which the UI element was dropped
        dropped_on: Option<Entity>,
    },
    /// Value of a UI element was changed by user input
    ValueChange,
    /// Value of a UI element was commited by user action
    ValueCommit,
    /// UI element gained focus
    Focus,
    /// UI element lost focus
    Blur,
}

/// Events that occur when the user interact with the UI
#[derive(Clone, Debug)]
pub struct UiEvent {
    /// The type of event
    pub event_type: UiEventType,
    /// The entity targeted by the event
    pub target: Entity,
}

impl TargetedEvent for UiEvent {
    fn target(&self) -> Entity {
        self.target
    }
}

impl UiEvent {
    /// Creates a new `UiEvent`.
    pub fn new(event_type: UiEventType, target: Entity) -> Self {
        Self { event_type, target }
    }
}

pub(crate) fn build_ui_mouse_system<T>(
    _world: &mut World,
    _resources: &mut Resources,
) -> Box<dyn Schedulable>
where
    T: BindingTypes,
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
                let mouse_position = mouse_world_position(mouse_position, &screen_dimensions);

                let targets = get_targeted(mouse_position, &sorted_widgets, world);

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

/// Returns the entities targeted by the cursor.
pub fn get_targeted<E>(
    (mouse_x, mouse_y): (f32, f32),
    sorted_widgets: &SortedWidgets,
    world: &E,
) -> HashSet<Entity>
where
    E: EntityStore,
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

/// Returns the entities target by the cursor below a Z value.
pub fn get_targeted_below<E>(
    (mouse_x, mouse_y): (f32, f32),
    below_global_z: f32,
    sorted_widgets: &SortedWidgets,
    world: &E,
) -> Option<Entity>
where
    E: EntityStore,
{
    let entities_below_z = sorted_widgets
        .widgets()
        .iter()
        .rev()
        .skip_while(|(_, z)| *z >= below_global_z)
        .map(|(e, _)| e);

    for &entity in entities_below_z {
        if let Some(transform) = world.get_component::<UiTransform>(entity) {
            if transform.opaque && transform.position_inside(mouse_x, mouse_y) {
                return Some(entity);
            }
        }
    }

    None
}

/// Returns the mouse position in UI world coordinates.
pub fn mouse_world_position(
    (mouse_x, mouse_y): (f32, f32),
    screen_dimensions: &ScreenDimensions,
) -> (f32, f32) {
    (
        mouse_x - screen_dimensions.width() / 2.0,
        screen_dimensions.height() - mouse_y - screen_dimensions.height() / 2.0,
    )
}
