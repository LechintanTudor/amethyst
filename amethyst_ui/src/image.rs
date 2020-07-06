use amethyst_rendy::palette::Srgba;

#[derive(Clone, Debug, PartialEq)]
pub enum UiImage {
    SolidColor(Srgba),
}