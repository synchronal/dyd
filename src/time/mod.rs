//! Functions for parsing time in different formats.
pub use relative::parse_relative;
pub use unix::parse_unix;

mod relative;
mod unix;
