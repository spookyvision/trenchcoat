// originally by https://github.com/smoltcp-rs/smoltcp/blob/master/src/macros.rs
#[cfg(not(test))]
#[cfg(feature = "log")]
macro_rules! trench_log {
    (trace, $($arg:expr),*) => { log::trace!($($arg),*) };
    (debug, $($arg:expr),*) => { log::debug!($($arg),*) };
    (info, $($arg:expr),*) => { log::info!($($arg),*) };
}

#[cfg(test)]
#[cfg(feature = "log")]
macro_rules! trench_log {
    (trace, $($arg:expr),*) => { println!($($arg),*) };
    (debug, $($arg:expr),*) => { println!($($arg),*) };
}

#[cfg(feature = "defmt")]
macro_rules! trench_log {
    (trace, $($arg:expr),*) => { defmt::trace!($($arg),*) };
    (debug, $($arg:expr),*) => { defmt::debug!($($arg),*) };
}

#[cfg(not(any(feature = "log", feature = "defmt")))]
macro_rules! trench_log {
    ($level:ident, $($arg:expr),*) => {{ $( let _ = $arg; )* }}
}

// TODO, later

macro_rules! trench_trace {
    ($($arg:expr),*) => (trench_log!(trace, $($arg),*));
}

macro_rules! trench_debug {
    ($($arg:expr),*) => (trench_log!(debug, $($arg),*));
}

macro_rules! trench_info {
    ($($arg:expr),*) => (trench_log!(info, $($arg),*));
}

// macro_rules! trench_trace {
//     ($($arg:expr),*) => {{}};
// }

// macro_rules! trench_debug {
//     ($($arg:expr),*) => {{}};
// }
