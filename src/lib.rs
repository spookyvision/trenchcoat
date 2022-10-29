#![cfg_attr(not(any(test, feature = "use-std")), no_std)]

#[macro_use]
mod macros;

pub mod forth;
pub mod pixelblaze;
pub mod vanillajs;

// TODO reduce this surface by building a better compiler wrapper
#[cfg(feature = "compiler")]
pub mod prelude {
    pub use phf;
    pub use postcard;
    pub use serde;
    pub use swc_common;
    pub use swc_ecma_parser;
    pub use swc_ecma_visit;
}
