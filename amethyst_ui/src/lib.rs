//! Provides components and systems to create an in game user interface.
#![allow(dead_code)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

pub use self::{
    bundle::UiBundle,
    layout::{Anchor, ScaleMode, Stretch},
    transform::UiTransform,
};

mod bundle;
mod layout;
mod transform;