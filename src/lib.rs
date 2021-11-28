//! # Gusket
//! [![GitHub actions](https://github.com/SOF3/gusket/workflows/CI/badge.svg)](https://github.com/SOF3/gusket/actions?query=workflow%3ACI)
//! [![crates.io](https://img.shields.io/crates/v/gusket.svg)](https://crates.io/crates/gusket)
//! [![crates.io](https://img.shields.io/crates/d/gusket.svg)](https://crates.io/crates/gusket)
//! [![docs.rs](https://docs.rs/gusket/badge.svg)](https://docs.rs/gusket)
//! [![GitHub](https://img.shields.io/github/last-commit/SOF3/gusket)](https://github.com/SOF3/gusket)
//! [![GitHub](https://img.shields.io/github/stars/SOF3/gusket?style=social)](https://github.com/SOF3/gusket)
//!
//! Gusket is a getter/setter derive macro.
//!
//! # Comparison with [`getset`](https://github.com/Hoverbear/getset):
//! - `gusket` only exposes one derive macro.
//! No need to `derive(Getters, MutGetters, Setters)` all the time.
//! This avoids accidentally forgetting some derives,
//! e.g. writing `#[getset(get_copy)]` with only `#[derive(getset::Getters)]`
//! will generate nothing without triggering a compile error.
//! - `gusket` uses the struct visibility by default.
//! This means that the usual boilerplate
//! `#[getset(get = "pub", get_mut = "pub", set = "pub")]`
//! is simplified to just `#[gusket]`.
//! - `gusket` generates code from the span of the field (not the derive call),
//! so error messages are more readable.

/// Derives getters and setters implementation for a struct.
///
/// # Example
/// ```rust
/// use gusket::Gusket;
///
/// #[derive(Gusket)]
/// #[gusket(immut)]
/// struct Alpha {
///     #[gusket]
///     foo:   String,
///     #[gusket(copy)]
///     bar:   u32,
///     #[gusket(copy, mut)]
///     qux:   i32,
///     #[gusket(mut, vis = pub(self))]
///     corge: Vec<u32>,
/// }
///
/// fn x(mut foo: Alpha) {
///     let _: &String = foo.foo();
///     let _: u32 = foo.bar();
///     let _: &mut i32 = foo.qux_mut();
///     foo.set_qux(0i32);
///     let _: &mut Vec<u32> = foo.corge_mut();
/// }
/// ```
///
/// See [`tests/full.rs`](https://docs.rs/crate/gusket/*/source/tests/full.rs)
/// for a full verification of the functionality.
pub use gusket_codegen::Gusket;
