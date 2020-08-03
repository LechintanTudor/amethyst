use crate::UiImage;
use amethyst_core::ecs::prelude::*;
use amethyst_rendy::palette::Srgba;

#[derive(Clone, Debug)]
pub struct UiButtonAction {
    pub target: Entity,
    pub event_type: UiButtonActionType,
}

impl UiButtonAction {
    pub fn new(target: Entity, event_type: UiButtonActionType) -> Self {
        Self { target, event_type }
    }
}

#[derive(Clone, Debug)]
pub enum UiButtonActionType {
    SetImage(UiImage),
    UnsetImage(UiImage),
    SetTextColor(Srgba),
    UnsetTextColor(Srgba),
}
