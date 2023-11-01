//! Array for sum types.

/// Different types of union layouts.
pub trait UnionType {}

/// The dense union layout.
#[derive(Clone, Copy)]
pub struct DenseLayout;

impl UnionType for DenseLayout {}

/// The sparse union layout.
#[derive(Clone, Copy)]
pub struct SparseLayout;

impl UnionType for SparseLayout {}

/// Indicates that a [`UnionType`] generic is not applicable.
///
/// This is used instead to prevent confusion in code because we don't have default
/// types for generic associated types.
///
/// This still shows up as [`DenseLayout`] in documentation but there is no way
/// to prevent that.
pub type NA = DenseLayout;
