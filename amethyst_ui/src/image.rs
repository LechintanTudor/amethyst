use amethyst_assets::Handle;
use amethyst_rendy::{
    SpriteRender, Texture,
    palette::Srgba,
};

#[derive(Clone, Debug, PartialEq)]
pub enum UiImage {
    Texture(Handle<Texture>),
    PartialTexture {
        texture: Handle<Texture>,
        left: f32,
        top: f32,
        right: f32,
        bottom: f32,
    },
    Sprite(SpriteRender),
    SolidColor(Srgba),
}