use amethyst_assets::{Asset, Format, ProcessableAsset, ProcessingState};
use amethyst_core::ecs::prelude::*;
use amethyst_error::{format_err, Error, ResultExt};
use glyph_brush::ab_glyph::FontArc;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct FontAsset(pub FontArc);

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
pub struct FontData(FontArc);

amethyst_assets::register_format_type!(FontData);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TtfFormat;

amethyst_assets::register_format!("TTF", TtfFormat as FontData);

impl Format<FontData> for TtfFormat {
    fn name(&self) -> &'static str {
        "TTF"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<FontData, Error> {
        FontArc::try_from_vec(bytes)
            .map(FontData)
            .with_context(|_| format_err!("Font parsing error"))
    }
}

pub fn build_font_asset_processor_system(
    world: &mut World,
    resources: &mut Resources,
) -> Box<dyn Schedulable> {
    amethyst_assets::build_asset_processor_system::<FontAsset>(world, resources)
}
