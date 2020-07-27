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
        entities: [entity]
        components: [
            (has UiTransform as transform on entity)
        ]
);