use serde::{Deserialize, Serialize};
use shrinkwraprs::Shrinkwrap;
use std::borrow::Cow;

/// A component that gives a name to an [`Entity`].
///
/// There are two ways you can get a name for an entity:
///
/// * Hard-coding the entity name in code, in which case the name would be a [`&'static str`][str].
/// * Dynamically generating the string or loading it from a data file, in which case the name
///   would be a `String`.
///
/// To support both of these cases smoothly, `Named` stores the name as [`Cow<'static, str>`].
/// You can pass either a [`&'static str`][str] or a [`String`] to [`Named::new`], and your code
/// can generally treat the `name` field as a [`&str`][str] without needing to know whether the
/// name is actually an owned or borrowed string.
///
/// [`Entity`]: https://docs.rs/specs/*/specs/struct.Entity.html
/// [`Cow<'static, str>`]: https://doc.rust-lang.org/std/borrow/enum.Cow.html
/// [`String`]: https://doc.rust-lang.org/std/string/struct.String.html
/// [str]: https://doc.rust-lang.org/std/primitive.str.html
/// [`Named::new`]: #method.new
#[derive(Shrinkwrap, Debug, Clone, Serialize, Deserialize)]
#[shrinkwrap(mutable)]
pub struct Named {
    /// The name of the entity this component is attached to.
    pub name: Cow<'static, str>,
}
