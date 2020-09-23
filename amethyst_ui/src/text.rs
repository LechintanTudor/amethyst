use crate::{Anchor, FontAsset};
use amethyst_assets::Handle;
use amethyst_rendy::palette::Srgba;

<<<<<<< HEAD
#[derive(Copy, Clone, Debug)]
pub(crate) struct CachedGlyph {
    pub x: f32,
    pub y: f32,
    pub advance_width: f32,
}
=======
use crate::Anchor;

use derivative::Derivative;
use serde::{Deserialize, Serialize};
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
use winit::{ElementState, Event, MouseButton, WindowEvent};

use amethyst_core::{
    ecs::prelude::{
        Component, DenseVecStorage, Join, Read, ReadExpect, ReadStorage, System, SystemData,
        WriteStorage,
    },
    shrev::{EventChannel, ReaderId},
    timing::Time,
};
use amethyst_derive::SystemDesc;
use amethyst_window::ScreenDimensions;

use super::*;
>>>>>>> origin/legion_v2

/// How lines should behave when they exceed their container's bounds
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LineMode {
    Single,
    Wrap,
}

/// Used to display text in the entity's `UiTransform`
#[derive(Clone)]
pub struct UiText {
    /// The text to be displayed
    pub text: String,
    /// Handle to the font that will be used to display the text
    pub font: Handle<FontAsset>,
    /// Font size in pixels
    pub font_size: f32,
    /// The color of the text
    pub color: Srgba,
    /// Whether the text should render normally or as dots
    pub password: bool,
    /// How to handle new lines
    pub line_mode: LineMode,
    /// The alignment of the text inside the `UiTransform`
    pub align: Anchor,
    pub(crate) cached_glyphs: Vec<CachedGlyph>,
}

impl UiText {
<<<<<<< HEAD
    /// Creates a new `UiText`.
    pub fn new<S>(font: Handle<FontAsset>, text: S, color: Srgba, font_size: f32) -> Self
    where
        S: ToString,
    {
        Self {
            text: text.to_string(),
=======
    /// Initializes a new UiText
    ///
    /// # Parameters
    ///
    /// * `font`: A handle to a `Font` asset
    /// * `text`: The glyphs to render
    /// * `color`: RGBA color with a maximum of 1.0 and a minimum of 0.0 for each channel
    /// * `font_size`: A uniform scale applied to the glyphs
    /// * `line_mode`: Text mode allowing single line or multiple lines
    /// * `align`: Text alignment within its `UiTransform`
    pub fn new(
        font: FontHandle,
        text: String,
        color: [f32; 4],
        font_size: f32,
        line_mode: LineMode,
        align: Anchor,
    ) -> UiText {
        UiText {
            text,
            color,
>>>>>>> origin/legion_v2
            font_size,
            color,
            font,
            password: false,
            line_mode,
            align,
            cached_glyphs: Vec::new(),
        }
    }
}
