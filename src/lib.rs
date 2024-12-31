#![cfg_attr(not(any(test, feature = "use-std")), no_std)]
#![feature(int_roundings)]
#[macro_use]
mod macros;

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod forth;
pub mod pixelblaze;
pub mod py;
pub mod vanillajs;

#[cfg(feature = "compiler")]
pub mod prelude {
    pub use phf;
    pub use postcard;
    pub use serde;
}

#[cfg(feature = "compiler")]
pub mod util {
    use std::collections::HashMap;

    pub trait PhfExt<V> {
        fn into_hashmap(self) -> HashMap<String, V>;
    }

    impl<K, V> PhfExt<V> for phf::Map<K, V>
    where
        K: ToString,
        V: Clone,
    {
        fn into_hashmap(self) -> HashMap<String, V> {
            self.into_iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect::<HashMap<_, _>>()
        }
    }
}
