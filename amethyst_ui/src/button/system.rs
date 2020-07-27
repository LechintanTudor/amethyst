use crate::{UiButtonAction, UiButtonActionType, UiImage, UiText};
use amethyst_core::{
    ecs::prelude::*,
    shrev::EventChannel,
};
use amethyst_rendy::palette::Srgba;
use std::{
    collections::HashMap,
    fmt::Debug,
};

#[derive(Debug)]
struct ActionChangeStack<T>
where
    T: Clone + PartialEq + Debug,
{
    initial_value: T,
    change_stack: Vec<T>,
}

impl<T> ActionChangeStack<T>
where
    T: Clone + PartialEq + Debug
{
    fn new(initial_value: T) -> Self {
        Self {
            initial_value,
            change_stack: Vec::new(),
        }
    }

    fn push(&mut self, change: T) {
        self.change_stack.push(change);
    }

    fn remove(&mut self, change: &T) {
        if let Some(i) = self.change_stack.iter().position(|c| c == change) {
            self.change_stack.remove(i);
        }
    }

    fn is_empty(&self) -> bool {
        self.change_stack.is_empty()
    }

    fn current_value(&self) -> T {
        if self.change_stack.is_empty() {
            self.initial_value.clone()
        } else {
            self.change_stack
            .iter()
            .rev()
            .next()
            .unwrap()
            .clone()
        }
    }
}

pub fn build_ui_button_system(world: &mut World, resources: &mut Resources) -> Box<dyn Schedulable> {
    let mut action_reader = resources
        .get_mut_or_default::<EventChannel<UiButtonAction>>()
        .unwrap()
        .register_reader();

    let mut text_color_map = HashMap::<Entity, ActionChangeStack<Srgba>>::new();
    let mut image_map = HashMap::<Entity, ActionChangeStack<UiImage>>::new();

    SystemBuilder::<()>::new("UiButtonSystem")
        .read_resource::<EventChannel<UiButtonAction>>()
        .write_component::<UiImage>()
        .write_component::<UiText>()
        .build(move|_, world, actions, _| {
            for action in actions.read(&mut action_reader) {
                match action.kind {
                    UiButtonActionType::SetTextColor(color) => {
                        if let Some(mut text) = world.get_component_mut::<UiText>(action.target) {
                            text_color_map
                                .entry(action.target)
                                .or_insert_with(|| ActionChangeStack::new(text.color))
                                .push(color);

                            text.color = color;
                        }
                    }
                    UiButtonActionType::UnsetTextColor(color) => {
                        if let Some(mut text) = world.get_component_mut::<UiText>(action.target) {
                            if let Some(text_color_stack) = text_color_map.get_mut(&action.target) {
                                text_color_stack.remove(&color);
                                text.color = text_color_stack.current_value();

                                if text_color_stack.is_empty() {
                                    text_color_map.remove(&action.target);
                                }
                            }
                        }
                    }
                    UiButtonActionType::SetImage(ref set_image) => {
                        if let Some(mut image) = world.get_component_mut::<UiImage>(action.target) {
                            image_map
                                .entry(action.target)
                                .or_insert_with(|| ActionChangeStack::new(image.clone()))
                                .push(set_image.clone());

                            *image = set_image.clone();
                        }
                    }
                    UiButtonActionType::UnsetImage(ref unset_image) => {
                        if let Some(mut image) = world.get_component_mut::<UiImage>(action.target) {
                            if let Some(image_stack) = image_map.get_mut(&action.target) {
                                image_stack.remove(unset_image);
                                *image = image_stack.current_value();

                                if image_stack.is_empty() {
                                    image_map.remove(&action.target);
                                }
                            }
                        }
                    }
                }
            }
        })
}