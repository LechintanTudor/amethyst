use amethyst_assets::Handle;
use amethyst_rendy::{palette::Srgba, SpriteRender, Texture};

/// Image used by UI elements, often as background
#[derive(Clone, Debug, PartialEq)]
pub enum UiImage {
    /// Display a texture.
    Texture(Handle<Texture>),
    /// Display part of a texture.
    PartialTexture {
        /// Texture handle
        texture: Handle<Texture>,
        /// Left texture coordinate
        left: f32,
        /// Top texture coordinate
        top: f32,
        /// Right texture coordinate
        right: f32,
        /// Bottom texture coordinate
        bottom: f32,
    },
    /// Display a sprite.
    Sprite(SpriteRender),
    /// Display a solid color.
    SolidColor(Srgba),
}
