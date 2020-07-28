//! Provides components and systems to create an in game user interface.
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

pub use self::{
    button::*,
    bundle::UiBundle,
    drag::Draggable,
    event::{UiEvent, UiEventType},
    event_retrigger::*,
    format::{FontAsset, TtfFormat},
    image::UiImage,
    layout::{Anchor, ScaleMode, Stretch},
    pass::RenderUi,
    selection::{Selected},
    sound::*,
    text::{LineMode, UiText, TextEditing},
    transform::UiTransform,
    widget::*,
};
pub use amethyst_core::ecs::entity::Entity;
pub use legion_transform::components::Parent;

mod button;
mod bundle;
mod drag;
mod event;
mod event_retrigger;
mod format;
mod glyphs;
mod image;
mod layout;
mod pass;
mod selection;
mod selection_order_cache;
mod sorted;
mod sound;
mod systems;
mod text;
mod text_editing;
mod transform;
mod utils;
mod widget;