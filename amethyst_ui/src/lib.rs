//! Provides components and systems to create an in game user interface.
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

pub use self::{
    bundle::UiBundle,
    format::{FontAsset, TtfFormat},
    layout::{Anchor, ScaleMode, Stretch},
    selection::{Selectable, Selected},
    transform::UiTransform,
};

mod bundle;
mod event;
mod format;
mod layout;
mod selection;
mod selection_order_cache;
mod systems;
mod transform;