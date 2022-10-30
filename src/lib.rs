#![cfg_attr(not(any(test, feature = "use-std")), no_std)]
#![feature(int_roundings)]
#[macro_use]
mod macros;

pub mod forth;
pub mod pixelblaze;
pub mod vanillajs;

#[cfg(feature = "compiler")]
pub mod prelude {
    pub use phf;
    pub use postcard;
    pub use serde;
}
