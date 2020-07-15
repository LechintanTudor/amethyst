//! Provides components and systems to create an in game user interface.
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

pub use self::{
    bundle::UiBundle,
    drag::Draggable,
    event::{UiEvent, UiEventType},
    format::{FontAsset, TtfFormat},
    image::UiImage,
    layout::{Anchor, ScaleMode, Stretch},
    pass::RenderUi,
    selection::{Selectable, Selected},
    text::{LineMode, UiText, TextEditing},
    transform::UiTransform,
};
pub use legion_transform::components::Parent;

mod button;
mod bundle;
mod drag;
mod event;
mod format;
mod glyphs;
mod image;
mod layout;
mod pass;
mod selection;
mod selection_order_cache;
mod sorted;
mod systems;
mod text;
mod transform;
mod utils;