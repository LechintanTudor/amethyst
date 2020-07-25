use crate::UiImage;
use amethyst_core::ecs::prelude::*;
use amethyst_rendy::palette::Srgba;

#[derive(Clone, Debug)]
pub struct UiButtonAction {
    pub target: Entity,
    pub kind: UiButtonActionType,
}

impl UiButtonAction {
    pub fn new(target: Entity, kind: UiButtonActionType) -> Self {
        Self { target, kind }
    }
}

#[derive(Clone, Debug)]
pub enum UiButtonActionType {
    SetImage(UiImage),
    UnsetImage(UiImage),
    SetTextColor(Srgba),
    UnsetTextColor(Srgba),
}