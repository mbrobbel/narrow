//! Struct layouts with typed field access and schema descriptors.

use core::fmt::Debug;

use crate::{
    buffer::{Buffer, BufferRef, VecBuffer},
    collection::{AllocError, Collection, CollectionAllocIn, CollectionRealloc},
    layout::{
        MemoryLayout,
        null::{Nulls, Unit},
    },
    length::Length,
    nullability::{NonNullable, Nullability},
};

/// Static field metadata, independent of Arrow FFI or buffer ownership.
///
/// Field order is defined by [`StructFields::FIELDS`]. The child type and its
/// nullability come from [`Field::Child`]; an exporter combines that child
/// schema with this descriptor. Tuple fields can use their decimal index as
/// the name. Metadata keys must be unique within each field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct FieldMetadata {
    /// Field name, including any rename selected by derive.
    pub name: &'static str,
    /// Application metadata represented as UTF-8 key/value pairs.
    pub metadata: &'static [(&'static str, &'static str)],
}

impl FieldMetadata {
    /// Describes a field with no additional metadata.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self {
            name,
            metadata: &[],
        }
    }

    /// Sets static metadata. Implementors must supply unique keys.
    #[must_use]
    pub const fn with_metadata(
        mut self,
        metadata: &'static [(&'static str, &'static str)],
    ) -> Self {
        self.metadata = metadata;
        self
    }
}

/// A collection of rows stored in an ordered set of field columns.
///
/// Implementors must keep every child at the row count, including after an
/// allocation error or panic. Construction from independent columns must
/// check their lengths. Keep children private and expose shared references;
/// mutation must coordinate all columns. Empty field collections must track
/// their row count explicitly.
///
/// Implement [`Field<I>`] for every index in `FIELDS` and no other index.
/// Each implementation must return the corresponding column. Descriptors must
/// have unique metadata keys; generated implementations should check those
/// when parsing attributes. These are
/// semantic contracts; unsafe code must not rely on them for memory safety.
/// A derive can generate this trait, typed accessors, borrowed row views,
/// checked constructors, and coordinated collection operations together.
pub trait StructFields: Collection {
    /// Ordered descriptors, one per child column.
    const FIELDS: &'static [FieldMetadata];

    /// Shared child references in descriptor order, typically a tuple.
    ///
    /// Keeping this type concrete lets consumers implement operations such as
    /// schema construction on the whole tuple, with their own per-child trait
    /// bounds, rather than erasing child types behind a runtime index.
    type Children<'a>
    where
        Self: 'a;

    /// Returns all raw child references in descriptor order.
    fn children(&self) -> Self::Children<'_>;
}

/// Typed, shared access to the field at index `I`.
///
/// The reference exposes the raw child, without applying parent validity.
/// A derive may also generate named accessors with the Rust field's
/// visibility. Generic access through this public trait exposes all fields
/// for types that implement it; it does not provide Rust field privacy.
/// Invalid indices are rejected through a missing trait implementation.
///
/// ```compile_fail,E0277
/// use narrow::layout::{null::Nulls, r#struct::Struct};
/// let rows = Struct::<Nulls>::from_buffer(Nulls::new(3));
/// let _ = rows.field::<0>(); // Empty structs have no child columns.
/// ```
pub trait Field<const I: usize>: StructFields {
    /// Physical child collection, retaining its concrete type and ownership.
    type Child: Collection;

    /// Returns the raw child column at index `I`.
    fn child(&self) -> &Self::Child;
}

impl<T: Unit> StructFields for Nulls<T> {
    const FIELDS: &'static [FieldMetadata] = &[];
    type Children<'a> = ();
    fn children(&self) {}
}

/// A struct array stored as a collection of field columns.
///
/// `Fields` supplies the row type, borrowed row views, and child storage. It
/// must keep every field column at the same length and preserve that invariant
/// during construction and mutation, including unwinding. A collection with no
/// fields must track its row count explicitly. This wrapper delegates those
/// responsibilities to `Fields`; it does not inspect individual columns.
///
/// `Nulls` controls validity of whole rows independently of nullable fields.
/// `Storage` selects the row validity storage; field storage is selected by
/// `Fields`. Nullable construction uses the existing [`crate::validity::Validity`]
/// capabilities, including default row values for null slots.
///
/// A manual [`crate::layout::Layout`] implementation can select
/// `Struct<MyFields<Storage>, Nulls, Storage>` to support `Array<MyRow>`.
/// Also implement [`crate::layout::NullableLayout`] for `MyRow` to enable
/// `Array<Option<MyRow>>`. Construction and `CollectionRealloc` operations
/// on nullable structs require `MyRow: Default`; read access does not.
///
/// # Examples
///
/// Empty structs use [`Nulls`] as a zero-field collection. The outer layout
/// makes these present struct rows, rather than Arrow null values.
///
/// ```
/// use narrow::{collection::Collection, layout::{null::Nulls, r#struct::Struct},
///     nullability::Nullable};
/// let rows = [Some(()), None].into_iter().collect::<Struct<Nulls, Nullable>>();
/// assert_eq!(rows.owned(0), Some(Some(())));
/// assert_eq!(rows.owned(1), Some(None));
/// assert!(Struct::<Nulls>::field_metadata().is_empty());
/// ```
pub struct Struct<
    Fields: StructFields,
    Nulls: Nullability = NonNullable,
    Storage: Buffer = VecBuffer,
>(Nulls::Collection<Fields, Storage>);

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> MemoryLayout
    for Struct<Fields, Nulls, Storage>
{
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> Struct<Fields, Nulls, Storage> {
    /// Returns ordered field descriptors without allocating or accessing data.
    #[must_use]
    pub fn field_metadata() -> &'static [FieldMetadata] {
        Fields::FIELDS
    }

    /// Wraps a field collection, including row validity when nullable.
    ///
    /// The field collection must already enforce equal child lengths.
    #[must_use]
    pub fn from_buffer(buffer: Nulls::Collection<Fields, Storage>) -> Self {
        Self(buffer)
    }

    /// Returns the field collection, including row validity when nullable.
    #[must_use]
    pub fn into_buffer(self) -> Nulls::Collection<Fields, Storage> {
        self.0
    }
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> BufferRef
    for Struct<Fields, Nulls, Storage>
{
    type Buffer = Nulls::Collection<Fields, Storage>;

    fn buffer_ref(&self) -> &Self::Buffer {
        &self.0
    }
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> Debug
    for Struct<Fields, Nulls, Storage>
where
    Nulls::Collection<Fields, Storage>: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Struct").field(&self.0).finish()
    }
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> Clone
    for Struct<Fields, Nulls, Storage>
where
    Nulls::Collection<Fields, Storage>: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> Default
    for Struct<Fields, Nulls, Storage>
where
    Nulls::Collection<Fields, Storage>: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> Extend<Nulls::Item<Fields::Owned>>
    for Struct<Fields, Nulls, Storage>
where
    Nulls::Collection<Fields, Storage>: Extend<Nulls::Item<Fields::Owned>>,
{
    fn extend<I: IntoIterator<Item = Nulls::Item<Fields::Owned>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer>
    FromIterator<Nulls::Item<Fields::Owned>> for Struct<Fields, Nulls, Storage>
where
    Nulls::Collection<Fields, Storage>: FromIterator<Nulls::Item<Fields::Owned>>,
{
    fn from_iter<I: IntoIterator<Item = Nulls::Item<Fields::Owned>>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> Length
    for Struct<Fields, Nulls, Storage>
{
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> Collection
    for Struct<Fields, Nulls, Storage>
{
    type View<'collection>
        = <Nulls::Collection<Fields, Storage> as Collection>::View<'collection>
    where
        Self: 'collection;

    type Owned = Nulls::Item<Fields::Owned>;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        self.0.view(index)
    }

    type Iter<'collection>
        = <Nulls::Collection<Fields, Storage> as Collection>::Iter<'collection>
    where
        Self: 'collection;

    fn iter_views(&self) -> Self::Iter<'_> {
        self.0.iter_views()
    }

    type IntoIter = <Nulls::Collection<Fields, Storage> as Collection>::IntoIter;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.0.into_iter_owned()
    }
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> CollectionAllocIn
    for Struct<Fields, Nulls, Storage>
where
    Nulls::Collection<Fields, Storage>: CollectionAllocIn,
{
    type Alloc = <Nulls::Collection<Fields, Storage> as CollectionAllocIn>::Alloc;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        Self(Nulls::Collection::<Fields, Storage>::with_capacity_in(
            capacity, alloc,
        ))
    }

    fn from_iter_in<I: IntoIterator<Item = Self::Owned>>(iter: I, alloc: Self::Alloc) -> Self {
        Self(Nulls::Collection::<Fields, Storage>::from_iter_in(
            iter, alloc,
        ))
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError> {
        Nulls::Collection::<Fields, Storage>::try_with_capacity_in(capacity, alloc).map(Self)
    }

    fn try_from_iter_in<I: IntoIterator<Item = Self::Owned>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError> {
        Nulls::Collection::<Fields, Storage>::try_from_iter_in(iter, alloc).map(Self)
    }
}

impl<Fields: StructFields, Nulls: Nullability, Storage: Buffer> CollectionRealloc
    for Struct<Fields, Nulls, Storage>
where
    Nulls::Collection<Fields, Storage>: CollectionRealloc,
{
    fn allocator(&self) -> Self::Alloc {
        self.0.allocator()
    }

    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        self.0.try_reserve(additional)
    }

    fn try_extend<I: IntoIterator<Item = Self::Owned>>(
        &mut self,
        iter: I,
    ) -> Result<(), AllocError> {
        self.0.try_extend(iter)
    }

    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }
}

impl<Fields: StructFields, N: Nullability, Storage: Buffer> Struct<Fields, N, Storage> {
    /// Returns raw fields without applying parent validity.
    ///
    /// After an interrupted nullable extension, raw columns can contain an
    /// uncommitted suffix beyond the visible row count. Use `field_view` to
    /// enforce the parent bounds and validity when projecting values.
    pub fn fields(&self) -> &Fields {
        N::collection_ref::<Fields, Storage>(&self.0)
    }

    /// Returns all raw child references with their concrete tuple type.
    ///
    /// See [`Self::fields`] for parent validity and uncommitted suffix semantics.
    pub fn children(&self) -> Fields::Children<'_> {
        self.fields().children()
    }

    /// Returns a typed raw child column, including values hidden by null rows.
    ///
    /// See [`Self::fields`] for the possible uncommitted suffix.
    pub fn field<const I: usize>(&self) -> &<Fields as Field<I>>::Child
    where
        Fields: Field<I>,
    {
        Field::<I>::child(self.fields())
    }

    /// Projects a row onto a field, preserving parent and child nullability.
    ///
    /// The outer `Option` bounds the row index. `N::Item` carries parent
    /// nullability; the child view carries the child's own nullability.
    /// For a nullable parent, `Some(None)` means a null row and the child is
    /// not accessed. A valid row returns `Some(Some(child_view))`.
    pub fn field_view<const I: usize>(
        &self,
        index: usize,
    ) -> Option<N::Item<<<Fields as Field<I>>::Child as Collection>::View<'_>>>
    where
        Fields: Field<I>,
    {
        N::project::<Fields, Storage, _, _>(&self.0, index, |fields| {
            Field::<I>::child(fields).view(index)
        })
    }
}
