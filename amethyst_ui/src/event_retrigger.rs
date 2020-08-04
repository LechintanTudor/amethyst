use crate::event::TargetedEvent;
use amethyst_core::{
    ecs::{prelude::*, storage::Component},
    shrev::{Event, EventChannel},
};
use std::ops::DerefMut;

/// Trait implemented by types which can receive events
pub trait EventReceiver<T> {
    /// Receive a single event
    fn receive_one(&mut self, value: &T);
    /// Receive a slice of events
    fn receive(&mut self, values: &[T]);
}

impl<T> EventReceiver<T> for EventChannel<T>
where
    T: Event + Clone,
{
    fn receive_one(&mut self, value: &T) {
        self.single_write(value.clone());
    }

    fn receive(&mut self, values: &[T]) {
        self.iter_write(values.iter().cloned());
    }
}

/// Trait that denotes which event gets retriggered to which other event and how
pub trait EventRetrigger
where
    Self: Component,
{
    /// The type of event that causes the retrigger
    type In: Send + Sync + Clone + TargetedEvent;
    /// The type of event that gets retriggered
    type Out: Send + Sync + Clone;

    /// Describes how `In` events retrigger `Out` events.
    fn apply<R>(&self, event: &Self::In, receiver: &mut R)
    where
        R: EventReceiver<Self::Out>;
}

pub(crate) fn build_event_retrigger_system<T>(
    _world: &mut World,
    resources: &mut Resources,
) -> Box<dyn Schedulable>
where
    T: EventRetrigger,
{
    let mut event_reader = resources
        .get_mut_or_default::<EventChannel<T::In>>()
        .unwrap()
        .register_reader();

    SystemBuilder::<()>::new("EventRetriggerSystem")
        .read_resource::<EventChannel<T::In>>()
        .write_resource::<EventChannel<T::Out>>()
        .read_component::<T>()
        .build(move |_, world, resources, _| {
            let (in_events, out_events) = resources;

            for event in in_events.read(&mut event_reader) {
                if let Some(retrigger) = world.get_component::<T>(event.target()) {
                    retrigger.apply(event, out_events.deref_mut());
                }
            }
        })
}
