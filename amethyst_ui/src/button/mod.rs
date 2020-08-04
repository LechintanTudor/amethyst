pub(crate) use self::system::*;
pub use self::{action::*, builder::*, retrigger::*};

mod action;
mod builder;
mod retrigger;
mod system;

use crate::{define_widget, UiTransform};

define_widget!(
    UiButton =>
        entities: [entity]
        components: [
            (has UiTransform as transform on entity)
        ]
);
