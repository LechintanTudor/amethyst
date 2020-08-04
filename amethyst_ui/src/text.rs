use crate::{Anchor, FontAsset};
use amethyst_assets::Handle;
use amethyst_rendy::palette::Srgba;

#[derive(Copy, Clone, Debug)]
pub(crate) struct CachedGlyph {
    pub x: f32,
    pub y: f32,
    pub advance_width: f32,
}

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
    /// Creates a new `UiText`.
    pub fn new<S>(font: Handle<FontAsset>, text: S, color: Srgba, font_size: f32) -> Self
    where
        S: ToString,
    {
        Self {
            text: text.to_string(),
            font_size,
            color,
            font,
            password: false,
            line_mode: LineMode::Single,
            align: Anchor::Middle,
            cached_glyphs: Vec::new(),
        }
    }
}
