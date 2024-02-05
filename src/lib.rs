//! # narrow
//!
//! An experimental (work-in-progress) implementation of [Apache Arrow](https://arrow.apache.org).

#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg",
    html_favicon_url = "https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg"
)]
// The goal of the list of lints here is to help reduce complexity and improve consistency
#![deny(
    // Rustc
    missing_copy_implementations,
    // missing_debug_implementations,
    missing_docs,
    noop_method_call,
    warnings,
    unused,
    // Clippy
    clippy::all,
    clippy::suspicious,
    clippy::style,
    clippy::complexity,
    clippy::perf,
    clippy::pedantic,
    // clippy::restrictions
    // clippy::arithmetic_side_effects, TODO(mbrobbel): check all
    clippy::as_conversions,
    clippy::as_underscore,
    clippy::clone_on_ref_ptr,
    clippy::decimal_literal_representation,
    clippy::empty_structs_with_brackets,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::missing_docs_in_private_items,
    clippy::multiple_unsafe_ops_per_block,
    clippy::pattern_type_mismatch,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::same_name_method,
    clippy::self_named_module_files,
    clippy::semicolon_outside_block,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::shadow_unrelated,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_to_string,
    clippy::tests_outside_test_module,
    clippy::undocumented_unsafe_blocks,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unseparated_literal_suffix,
    clippy::unwrap_used,
    // Rustdoc
    rustdoc::all
)]
#![allow(clippy::module_name_repetitions, clippy::pub_use)]

mod fixed_size;
pub use self::fixed_size::FixedSize;

mod length;
pub use self::length::Length;

mod index;
pub use self::index::Index;

pub mod buffer;

pub mod bitmap;

pub(crate) mod nullable;
// TODO(mbrobbel): pub(crate)
pub mod offset;
pub(crate) mod validity;

pub mod array;

pub mod logical;

#[cfg(feature = "arrow-rs")]
pub mod arrow;

// Re-export `narrow_derive` macros when the `derive` feature is enabled.
#[cfg(feature = "derive")]
pub use narrow_derive::ArrayType;

// This allows using the `ArrayType` derive macro in tests.
#[cfg(all(test, feature = "derive"))]
extern crate self as narrow;
