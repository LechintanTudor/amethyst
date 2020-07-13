use crate::{
    Anchor, FontAsset,
};
use amethyst_assets::Handle;
use amethyst_rendy::palette::Srgba;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug)]
pub(crate) struct CachedGlyph {
    pub x: f32,
    pub y: f32,
    pub advance_width: f32,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum LineMode {
    Single,
    Wrap,
}

#[derive(Clone)]
pub struct UiText {
    pub text: String,
    pub font: Handle<FontAsset>,
    pub font_size: f32,
    pub color: Srgba,
    pub password: bool,
    pub line_mode: LineMode,
    pub align: Anchor,
    pub(crate) cached_glyphs: Vec<CachedGlyph>,
}

impl UiText {
    pub fn new(font: Handle<FontAsset>, text: String, color: Srgba, font_size: f32) -> Self {
        Self {
            text,
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

#[derive(Copy, Clone, Debug)]
pub struct TextEditing {
    pub cursor_position: isize,
    pub max_length: usize,
    pub highlight_vector: isize,
    pub selected_text_color: Srgba,
    pub selected_background_color: Srgba,
    pub use_block_cursor: bool,
    pub(crate) cursor_blink_timer: f32,
}

impl TextEditing {
    pub fn new(
        max_length: usize,
        selected_text_color: Srgba,
        selected_background_color: Srgba,
        use_block_cursor: bool,
    ) -> Self
    {
        Self {
            cursor_position: 0,
            max_length,
            highlight_vector: 0,
            selected_text_color,
            selected_background_color,
            use_block_cursor,
            cursor_blink_timer: 0.0,
        }
    }
}