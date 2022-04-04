//! This crate provides a convenient concise way to write unit tests for
//! implementations of [`CloneOr`].
extern crate clone_or;
#[cfg_attr(test, macro_use)]
extern crate clone_or_derive;
mod core;
mod nested;