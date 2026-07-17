#![no_std]
#![cfg_attr(not(feature = "derive"), doc = "# Narrow")]
#![cfg_attr(feature = "derive", doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md")))]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg",
    html_favicon_url = "https://raw.githubusercontent.com/mbrobbel/narrow/main/narrow.svg"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// The goal of the list of lints here is to help reduce complexity and improve consistency
#![deny(
    // Rustc
    missing_copy_implementations,
    missing_debug_implementations,
    // missing_docs,
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
    // clippy::restriction,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::as_underscore,
    clippy::clone_on_ref_ptr,
    clippy::decimal_literal_representation,
    clippy::empty_structs_with_brackets,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    // clippy::missing_docs_in_private_items,
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
#![forbid(unsafe_code)]
#![allow(
    clippy::into_iter_without_iter,
    clippy::iter_not_returning_iterator,
    clippy::module_name_repetitions,
    clippy::pub_use,
    unsafe_op_in_unsafe_fn
)]

pub mod collection;
pub mod fixed_size;
pub mod length;

pub mod buffer;

pub mod bitmap;

pub mod nullability;
pub mod validity;

pub mod offset;

pub mod layout;

pub mod array;
