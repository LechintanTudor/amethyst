use crate::event::TargetedEvent;
use amethyst_core::{
    ecs::{prelude::*, storage::Component},
    shrev::{Event, EventChannel},
};
use std::ops::DerefMut;

pub trait EventReceiver<T> {
    fn receive_one(&mut self, value: &T);

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

pub trait EventRetrigger
where
    Self: Component,
{
    type In: Send + Sync + Clone + TargetedEvent;
    type Out: Send + Sync + Clone;

    fn apply<R>(&self, event: &Self::In, receiver: &mut R)
    where
        R: EventReceiver<Self::Out>;
}

pub fn build_event_retrigger_system<T>(
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
