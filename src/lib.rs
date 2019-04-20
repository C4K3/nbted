#![warn(
    unused_results,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences,
    trivial_casts,
    trivial_numeric_casts
)]
#[macro_use]
extern crate failure;

pub type Result<T> = std::result::Result<T, failure::Error>;

pub mod data;
pub mod iter_replacer;
pub mod read;
pub mod string_read;
pub mod string_write;
pub mod write;

#[cfg(test)]
mod tests;
