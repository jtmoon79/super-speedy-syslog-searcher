#![doc = include_str!("../README.md")]

pub use ::ere_core::*;
pub use ::ere_macros::*;

/// Includes the basic things you'll need.
///
/// Unless you want to use a specific engine or more specific internals,
/// you will probably never need anything else.
pub mod prelude {
    pub use ::ere_core::Regex;
    pub use ::ere_macros::compile_regex;
    #[cfg(feature = "unstable-attr-regex")]
    pub use ::ere_macros::regex;
}
