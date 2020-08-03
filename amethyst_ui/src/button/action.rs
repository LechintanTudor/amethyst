use crate::UiImage;
use amethyst_core::ecs::prelude::*;
use amethyst_rendy::palette::Srgba;

/// Describes an action targeted at an `UiButton`.
#[derive(Clone, Debug)]
pub struct UiButtonAction {
    /// The type of action
    pub event_type: UiButtonActionType,
    /// The targeted entity
    pub target: Entity,
}

impl UiButtonAction {
    /// Creates a new `UiButtonAction`.
    pub fn new(event_type: UiButtonActionType, target: Entity) -> Self {
        Self { event_type, target }
    }
}

/// Describes the type of a `UiButtonAction`.
#[derive(Clone, Debug)]
pub enum UiButtonActionType {
    /// Sets the texture of a `UiButton` to the given `UiImage`.
    SetImage(UiImage),
    /// Removes a previously set `UiImage` on a `UiButton`.
    UnsetImage(UiImage),
    /// Sets the text color of a `UiButton`.
    SetTextColor(Srgba),
    /// Removes a previously set text color on a `UiButton`.
    UnsetTextColor(Srgba),
}
