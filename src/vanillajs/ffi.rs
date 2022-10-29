use super::runtime::VanillaJSFFI;

#[cfg(feature = "compiler")]
pub const FFI_FUNCS: phf::Map<&'static str, VanillaJSFFI> = phf::phf_map! {
    "console_log" => VanillaJSFFI::ConsoleLog,
    "math_pow" => VanillaJSFFI::MathPow,
};
