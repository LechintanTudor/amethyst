use amethyst_assets::{Asset, Format, ProcessableAsset, ProcessingState};
use amethyst_error::{format_err, Error, ResultExt};
use glyph_brush::rusttype::Font;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct FontAsset(pub Font<'static>);

impl Asset for FontAsset {
    const NAME: &'static str = "ui::Font";
    type Data = FontData;
}

impl ProcessableAsset for FontAsset {
    fn process(data: FontData) -> Result<ProcessingState<FontAsset>, Error> {
        Ok(ProcessingState::Loaded(FontAsset(data.0)))
    }
}

#[derive(Clone)]
pub struct FontData(Font<'static>);

amethyst_assets::register_format_type!(FontData);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtfFormat;

amethyst_assets::register_format!("TTF", TtfFormat as FontData);

impl Format<FontData> for TtfFormat {
    fn name(&self) -> &'static str {
        "TTF"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<FontData, Error> {
        Font::from_bytes(bytes)
            .map(FontData)
            .with_context(|_| format_err!("Font parsing error"))
    }
}