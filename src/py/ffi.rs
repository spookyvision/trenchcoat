use crate::pixelblaze::ffi::PixelBlazeFFI;

#[cfg(feature = "compiler")]
pub const FFI_FUNCS: phf::Map<&'static str, PixelBlazeFFI> = phf::phf_map! {
    "print" => PixelBlazeFFI::ConsoleLog,
};
