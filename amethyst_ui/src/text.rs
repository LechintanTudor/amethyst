use crate::{
    Anchor, FontAsset,
};
use amethyst_assets::Handle;
use amethyst_rendy::palette::Srgba;

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
    pub fn new<S>(font: Handle<FontAsset>, text: S, color: Srgba, font_size: f32) -> Self
    where
        S: ToString
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
