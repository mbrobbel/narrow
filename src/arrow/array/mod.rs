//! Interop with [`arrow-array`].

mod boolean;
mod fixed_size_list;
mod fixed_size_primitive;
mod string;
mod r#struct;
pub use r#struct::StructArrayTypeFields;
mod variable_size_list;
