//! This was originally just a binary program, it was not meant to be used as a
//! library. As such, the library functionality here has been hidden inside
//! the "unstable" module. Use this module only with the understanding that
//! the library is not 1.0 stable (only the binary is.)

#![warn(
    unused_results,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences,
    trivial_casts,
    trivial_numeric_casts
)]

pub type Result<T> = std::result::Result<T, anyhow::Error>;

pub mod unstable;

use unstable::*;
