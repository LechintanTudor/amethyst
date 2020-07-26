pub use self::{
    action::*,
    builder::*,
    retrigger::*,
    system::*,
};

mod action;
mod builder;
mod retrigger;
mod system;

use crate::{
    define_widget,
    UiTransform,
};

define_widget!(
    UiButton =>
        entities: [text_entity, image_entity]
        components: [
            (has UiTransform as transform on image_entity),
            (has UiTransform as text_transform on text_entity)
        ]
);