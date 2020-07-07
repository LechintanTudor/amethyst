use amethyst_assets::Handle;
use amethyst_rendy::{
    Texture,
    palette::Srgba,
};

#[derive(Clone, Debug, PartialEq)]
pub enum UiImage {
    SolidColor(Srgba),
    Texture(Handle<Texture>),
}