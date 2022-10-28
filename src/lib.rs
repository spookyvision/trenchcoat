// #![cfg_attr(not(any(test, feature = "use-std")), no_std)]

pub mod forth;
pub mod pixelblaze;

#[cfg(feature = "parse")]
pub mod prelude {
    pub use swc_common;
    pub use swc_ecma_parser;
    pub use swc_ecma_visit;
}
