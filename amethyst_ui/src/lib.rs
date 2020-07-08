//! Provides components and systems to create an in game user interface.
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

pub use self::{
    bundle::UiBundle,
    event::UiEvent,
    format::{FontAsset, TtfFormat},
    image::UiImage,
    layout::{Anchor, ScaleMode, Stretch},
    pass::RenderUi,
    selection::{Selectable, Selected},
    transform::UiTransform,
};

mod bundle;
mod event;
mod format;
mod glyphs;
mod image;
mod layout;
mod pass;
mod selection;
mod selection_order_cache;
mod systems;
mod text;
mod transform;