use narrow::{
    array::Array,
    bitmap::{Bitmap, ValidityBitmap},
    buffer::{Buffer, BufferRef, SliceBuffer},
    collection::{
        AllocError, ChildRef, Collection, CollectionAllocIn, CollectionRealloc, owned::IntoOwned,
    },
    layout::{
        ArrayItem, Layout, NullableLayout,
        null::{Nulls, Unit},
        r#struct::{Field, FieldMetadata, Struct, StructFields},
    },
    length::Length,
    nullability::{Nullability, Nullable},
    validity::Validity,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Row<A, B> {
    first: A,
    second: B,
}

impl<A, B> Row<A, B> {
    fn from_tuple((first, second): (A, B)) -> Self {
        Self { first, second }
    }
}

impl<A: IntoOwned<C>, B: IntoOwned<D>, C, D> IntoOwned<Row<C, D>> for Row<A, B> {
    fn into_owned(self) -> Row<C, D> {
        Row {
            first: self.first.into_owned(),
            second: self.second.into_owned(),
        }
    }
}

impl<A: ArrayItem, B: ArrayItem> Layout for Row<A, B> {
    type Memory<Nulls: Nullability, Storage: Buffer> =
        Struct<Fields<A::Memory<Storage>, B::Memory<Storage>>, Nulls, Storage>;
}

impl<A: ArrayItem, B: ArrayItem> NullableLayout for Row<A, B> {}

/// Test field collection with private columns and checked construction.
#[derive(Clone, Debug, Default)]
struct Fields<A, B> {
    first: A,
    second: B,
    max_rows: Option<usize>,
    panic_after_first: bool,
}

impl<A: Collection, B: Collection> StructFields for Fields<A, B> {
    const FIELDS: &'static [FieldMetadata] = &[
        FieldMetadata::new("first").with_metadata(&[("description", "first column")]),
        FieldMetadata::new("second"),
    ];
    type Children<'a>
        = (&'a A, &'a B)
    where
        Self: 'a;
    fn children(&self) -> Self::Children<'_> {
        (&self.first, &self.second)
    }
}

impl<A: Collection, B: Collection> Field<0> for Fields<A, B> {
    type Child = A;
    fn child(&self) -> &A {
        &self.first
    }
}

impl<A: Collection, B: Collection> Field<1> for Fields<A, B> {
    type Child = B;
    fn child(&self) -> &B {
        &self.second
    }
}

impl<A: Collection, B: Collection> Fields<A, B> {
    fn first(&self) -> &A {
        &self.first
    }

    fn try_new(first: A, second: B) -> Result<Self, &'static str> {
        if first.len() != second.len() {
            return Err("field lengths differ");
        }
        Ok(Self {
            first,
            second,
            max_rows: None,
            panic_after_first: false,
        })
    }
}

impl<A: Collection, B: Collection> Length for Fields<A, B> {
    fn len(&self) -> usize {
        self.first.len()
    }
}

impl<A: Collection, B: Collection> Collection for Fields<A, B> {
    type View<'a>
        = Row<A::View<'a>, B::View<'a>>
    where
        Self: 'a;
    type Owned = Row<A::Owned, B::Owned>;

    fn view(&self, index: usize) -> Option<Self::View<'_>> {
        Some(Row {
            first: self.first.view(index)?,
            second: self.second.view(index)?,
        })
    }

    type Iter<'a>
        = std::iter::Map<
        std::iter::Zip<A::Iter<'a>, B::Iter<'a>>,
        fn((A::View<'a>, B::View<'a>)) -> Self::View<'a>,
    >
    where
        Self: 'a;

    fn iter_views(&self) -> Self::Iter<'_> {
        self.first
            .iter_views()
            .zip(self.second.iter_views())
            .map(Row::from_tuple)
    }

    type IntoIter = std::iter::Map<
        std::iter::Zip<A::IntoIter, B::IntoIter>,
        fn((A::Owned, B::Owned)) -> Self::Owned,
    >;

    fn into_iter_owned(self) -> Self::IntoIter {
        self.first
            .into_iter_owned()
            .zip(self.second.into_iter_owned())
            .map(Row::from_tuple)
    }
}

impl<A, B> CollectionAllocIn for Fields<A, B>
where
    A: CollectionRealloc,
    B: CollectionRealloc<Alloc = A::Alloc>,
{
    type Alloc = A::Alloc;

    fn with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Self {
        Self {
            first: A::with_capacity_in(capacity, alloc.clone()),
            second: B::with_capacity_in(capacity, alloc),
            max_rows: None,
            panic_after_first: false,
        }
    }

    fn try_with_capacity_in(capacity: usize, alloc: Self::Alloc) -> Result<Self, AllocError> {
        Ok(Self {
            first: A::try_with_capacity_in(capacity, alloc.clone())?,
            second: B::try_with_capacity_in(capacity, alloc)?,
            max_rows: None,
            panic_after_first: false,
        })
    }

    fn from_iter_in<I: IntoIterator<Item = Row<A::Owned, B::Owned>>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Self {
        let mut fields = Self::with_capacity_in(0, alloc);
        fields.extend(iter);
        fields
    }

    fn try_from_iter_in<I: IntoIterator<Item = Row<A::Owned, B::Owned>>>(
        iter: I,
        alloc: Self::Alloc,
    ) -> Result<Self, AllocError> {
        let mut fields = Self::try_with_capacity_in(0, alloc)?;
        fields.try_extend(iter)?;
        Ok(fields)
    }
}

impl<A, B> CollectionRealloc for Fields<A, B>
where
    A: CollectionRealloc,
    B: CollectionRealloc<Alloc = A::Alloc>,
{
    fn allocator(&self) -> Self::Alloc {
        self.first.allocator()
    }

    fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        self.first.try_reserve(additional)?;
        self.second.try_reserve(additional)
    }

    fn try_extend<I: IntoIterator<Item = Row<A::Owned, B::Owned>>>(
        &mut self,
        iter: I,
    ) -> Result<(), AllocError> {
        let len = self.len();
        let mut rollback = FieldsRollback {
            fields: self,
            len: Some(len),
        };
        for row in iter {
            rollback.fields.first.try_extend([row.first])?;
            assert!(!rollback.fields.panic_after_first, "second child panicked");
            if rollback.fields.max_rows == Some(rollback.fields.second.len()) {
                return Err(AllocError);
            }
            rollback.fields.second.try_extend([row.second])?;
        }
        rollback.len = None;
        Ok(())
    }

    fn reserve(&mut self, additional: usize) {
        self.first.reserve(additional);
        self.second.reserve(additional);
    }

    fn truncate(&mut self, len: usize) {
        self.first.truncate(len);
        self.second.truncate(len);
    }
}

/// Restores both columns if an extension fails or unwinds.
struct FieldsRollback<'a, A, B>
where
    A: CollectionRealloc,
    B: CollectionRealloc<Alloc = A::Alloc>,
{
    fields: &'a mut Fields<A, B>,
    len: Option<usize>,
}

impl<A, B> Drop for FieldsRollback<'_, A, B>
where
    A: CollectionRealloc,
    B: CollectionRealloc<Alloc = A::Alloc>,
{
    fn drop(&mut self) {
        if let Some(len) = self.len {
            self.fields.truncate(len);
        }
    }
}

impl<A, B> Extend<Row<A::Owned, B::Owned>> for Fields<A, B>
where
    A: CollectionRealloc,
    B: CollectionRealloc<Alloc = A::Alloc>,
{
    fn extend<I: IntoIterator<Item = Row<A::Owned, B::Owned>>>(&mut self, iter: I) {
        self.try_extend(iter).expect("field allocation succeeds");
    }
}

impl<A, B> FromIterator<Row<A::Owned, B::Owned>> for Fields<A, B>
where
    A: CollectionRealloc<Alloc: Default>,
    B: CollectionRealloc<Alloc = A::Alloc>,
{
    fn from_iter<I: IntoIterator<Item = Row<A::Owned, B::Owned>>>(iter: I) -> Self {
        Self::from_iter_in(iter, Default::default())
    }
}

#[test]
fn columnar_rows_and_nested_views() {
    let input = [
        Row {
            first: 1,
            second: vec![Some(2), None],
        },
        Row {
            first: 3,
            second: vec![],
        },
    ];
    let rows = input
        .clone()
        .into_iter()
        .collect::<Array<Row<i32, Vec<Option<i32>>>>>();
    assert_eq!(rows.len(), 2);
    let view = rows.view(0).expect("first row");
    assert_eq!(view.first, 1);
    assert_eq!(view.second.owned(1), Some(None));
    assert!(rows.view(2).is_none());
    let fields = rows.buffer_ref().buffer_ref();
    assert_eq!(fields.first.buffer_ref(), &[1, 3]);
    assert_eq!(
        rows.iter_views()
            .map(IntoOwned::into_owned)
            .collect::<Vec<_>>(),
        input
    );
    let restored = Array::<Row<i32, Vec<Option<i32>>>>::from_buffer(rows.into_buffer());
    assert_eq!(restored.into_iter_owned().collect::<Vec<_>>(), input);
}

#[test]
fn nullable_rows_keep_field_nulls_independent() {
    let input = [
        Some(Row {
            first: 1,
            second: None::<i32>,
        }),
        None,
        Some(Row {
            first: 3,
            second: Some(4),
        }),
    ];
    let mut rows = input
        .into_iter()
        .collect::<Array<Option<Row<i32, Option<i32>>>>>();
    assert_eq!(rows.owned(0), Some(input[0]));
    assert_eq!(rows.owned(1), Some(None));
    assert_eq!(rows.owned(3), None);
    assert_eq!(rows.buffer_ref().buffer_ref().null_count(), 1);
    assert_eq!(
        rows.buffer_ref()
            .buffer_ref()
            .child_ref()
            .second
            .buffer_ref()
            .null_count(),
        2
    );
    rows.truncate(2);
    rows.try_extend([input[2]]).expect("extend");
    let mut owned = rows.into_iter_owned();
    assert_eq!(owned.len(), input.len());
    assert_eq!(owned.size_hint(), (input.len(), Some(input.len())));
    assert_eq!(owned.next(), Some(input[0]));
    assert_eq!(owned.size_hint(), (input.len() - 1, Some(input.len() - 1)));
    assert_eq!(owned.collect::<Vec<_>>(), input[1..]);
}

#[test]
fn allocation_and_mutation() {
    type Rows = Array<Row<i32, i32>>;
    let input = [
        Row {
            first: 7,
            second: 9,
        },
        Row {
            first: 8,
            second: 10,
        },
    ];
    let mut rows = Rows::with_capacity_in(2, ());
    assert!(rows.is_empty());
    rows.reserve(2);
    rows.extend(input);
    rows.try_reserve(1).expect("reserve");
    assert_eq!(rows.iter_views().collect::<Vec<_>>(), input);
    rows.truncate(1);
    assert_eq!(rows.into_iter_owned().len(), 1);
    assert_eq!(Rows::from_iter_in(input, ()).owned(1), Some(input[1]));
    let fallible = Rows::try_from_iter_in(input, ()).expect("allocation");
    assert_eq!(fallible.owned(0), Some(input[0]));
    assert!(Rows::try_with_capacity_in(usize::MAX, ()).is_err());
    assert!(Rows::default().is_empty());
}

#[test]
fn borrowed_columns_and_checked_validity() {
    let first = [1_i32, 2];
    let second = [3_i32, 4];
    let fields = Fields::try_new(first.as_slice(), second.as_slice()).expect("equal lengths");
    let rows = Struct::<_, narrow::nullability::NonNullable, SliceBuffer<'_>>::from_buffer(fields);
    assert_eq!(
        rows.owned(1),
        Some(Row {
            first: 2,
            second: 4
        })
    );
    assert_eq!(rows.buffer_ref().first.as_ptr(), first.as_ptr());
    assert_eq!(rows.clone().owned(0), rows.owned(0));
    let restored = Struct::<_>::from_buffer(rows.into_buffer());
    assert_eq!(restored.len(), 2);
    assert!(Fields::try_new(first.as_slice(), &second[..1]).is_err());

    let validity_bytes = [0b0000_0001_u8];
    let bitmap = Bitmap::<SliceBuffer<'_>>::try_from_parts(&validity_bytes, 2, 0)
        .expect("valid borrowed bitmap");
    let validity =
        Validity::try_from_parts(restored.into_buffer(), bitmap).expect("matching validity length");
    let nullable = Struct::<_, Nullable, SliceBuffer<'_>>::from_buffer(validity);
    assert_eq!(nullable.owned(1), Some(None));
    assert!(nullable.view(2).is_none());
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct EmptyRow;

impl IntoOwned<EmptyRow> for EmptyRow {
    fn into_owned(self) -> EmptyRow {
        self
    }
}

impl Unit for EmptyRow {}

impl Layout for EmptyRow {
    type Memory<N: Nullability, S: Buffer> = Struct<Nulls<Self>, N, S>;
}
impl NullableLayout for EmptyRow {}

#[test]
fn zero_fields_preserve_row_count() {
    let rows = Struct::<_>::from_buffer(Nulls::<EmptyRow>::new(3));
    assert_eq!(rows.len(), 3);
    assert_eq!(rows.view(2), Some(EmptyRow));
    assert_eq!(rows.view(3), None);
    assert_eq!(rows.iter_views().count(), 3);
    assert_eq!(rows.into_iter_owned().len(), 3);

    let empty = Struct::<_>::from_buffer(Nulls::<EmptyRow>::new(0));
    assert!(empty.is_empty());
    assert_eq!(empty.view(0), None);
}

#[test]
fn failed_extension_preserves_rows() {
    let mut fields = Fields::try_new(vec![1_i32], vec![2_i32]).expect("equal lengths");
    fields.max_rows = Some(2);
    let mut rows = Struct::<_>::from_buffer(fields);
    assert_eq!(
        rows.try_extend([
            Row {
                first: 3,
                second: 4
            },
            Row {
                first: 5,
                second: 6
            },
        ]),
        Err(AllocError)
    );
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows.owned(0),
        Some(Row {
            first: 1,
            second: 2
        })
    );
    assert_eq!(rows.buffer_ref().second.len(), 1);
    rows.try_extend([Row {
        first: 7,
        second: 8,
    }])
    .expect("capacity available after rollback");
    assert_eq!(
        rows.owned(1),
        Some(Row {
            first: 7,
            second: 8
        })
    );
}

#[test]
fn panicking_extension_preserves_columns() {
    let fields = Fields::try_new(vec![1_i32], vec![2_i32]).expect("equal lengths");
    let mut rows = Struct::<_>::from_buffer(fields);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rows.extend(
            [Row {
                first: 3,
                second: 4,
            }]
            .into_iter()
            .chain(std::iter::once_with(|| panic!("iterator failed"))),
        );
    }));
    assert!(result.is_err());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows.buffer_ref().second.len(), 1);
    rows.extend([Row {
        first: 5,
        second: 6,
    }]);
    assert_eq!(
        rows.owned(1),
        Some(Row {
            first: 5,
            second: 6
        })
    );
}

#[test]
fn typed_fields_and_metadata() {
    type Rows = Array<Row<i32, bool>>;
    let rows = [Row {
        first: 3,
        second: true,
    }]
    .into_iter()
    .collect::<Rows>();
    let layout = rows.buffer_ref();
    let first: &narrow::layout::fixed_size_primitive::FixedSizePrimitive<i32> = layout.field::<0>();
    let second: &narrow::layout::boolean::Boolean = layout.field::<1>();
    assert_eq!(first.owned(0), Some(3));
    assert_eq!(second.owned(0), Some(true));
    assert!(core::ptr::eq(first, layout.fields().first()));
    assert_eq!(layout.field_view::<0>(0), Some(3));
    assert_eq!(layout.field_view::<0>(1), None);
    let descriptors = Struct::<Fields<Vec<i32>, Vec<bool>>>::field_metadata();
    assert_eq!(descriptors.len(), 2);
    assert_eq!(descriptors[0].name, "first");
    assert_eq!(descriptors[0].metadata, &[("description", "first column")]);
    assert_eq!(descriptors[1].name, "second");
}

#[test]
fn projection_preserves_parent_and_child_nulls() {
    let fields = Fields::try_new(
        vec![7, 42, 9],
        [None, Some(5), Some(8)]
            .into_iter()
            .collect::<Array<Option<i32>>>(),
    )
    .unwrap();
    let validity =
        Validity::try_from_parts(fields, [true, false, true].into_iter().collect::<Bitmap>())
            .unwrap();
    let rows = Struct::<_, Nullable>::from_buffer(validity);
    assert_eq!(rows.field::<0>().owned(1), Some(42));
    assert_eq!(rows.field::<1>().owned(1), Some(Some(5)));
    assert_eq!(rows.field_view::<0>(1), Some(None));
    assert_eq!(rows.field_view::<1>(0), Some(Some(None)));
    assert_eq!(rows.field_view::<1>(1), Some(None));
    assert_eq!(rows.field_view::<1>(2), Some(Some(Some(8))));
    assert_eq!(rows.field_view::<1>(3), None);
}

#[test]
fn optional_empty_structs() {
    let mut rows = [Some(EmptyRow), None, Some(EmptyRow)]
        .into_iter()
        .collect::<Array<Option<EmptyRow>>>();
    assert_eq!(rows.owned(0), Some(Some(EmptyRow)));
    assert_eq!(rows.owned(1), Some(None));
    assert!(Struct::<Nulls<EmptyRow>>::field_metadata().is_empty());
    rows.truncate(1);
    rows.try_extend([None, Some(EmptyRow)]).unwrap();
    assert_eq!(
        rows.into_iter_owned().collect::<Vec<_>>(),
        [Some(EmptyRow), None, Some(EmptyRow)]
    );
}

#[test]
fn reading_nullable_non_default_rows() {
    #[derive(Clone, Copy, Debug, PartialEq)]
    struct NoDefault(i32);
    impl IntoOwned<Self> for NoDefault {
        fn into_owned(self) -> Self {
            self
        }
    }
    impl<'a> narrow::collection::view::AsView<'a> for NoDefault {
        type View = Self;
        fn as_view(&'a self) -> Self {
            *self
        }
    }
    let fields = Fields::try_new(vec![NoDefault(4), NoDefault(5)], vec![1, 2]).unwrap();
    let validity =
        Validity::try_from_parts(fields, [true, false].into_iter().collect::<Bitmap>()).unwrap();
    let rows = Struct::<_, Nullable>::from_buffer(validity);
    assert_eq!(
        rows.owned(0),
        Some(Some(Row {
            first: NoDefault(4),
            second: 1
        }))
    );
    assert_eq!(rows.owned(1), Some(None));
    assert_eq!(rows.field_view::<0>(1), Some(None));
}

#[test]
fn panic_between_child_appends_rolls_back() {
    let mut fields = Fields::try_new(vec![1_i32], vec![2_i32]).unwrap();
    fields.panic_after_first = true;
    let mut rows = Struct::<_>::from_buffer(fields);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rows.extend([Row {
            first: 3,
            second: 4,
        }]);
    }));
    assert!(result.is_err());
    assert_eq!(rows.field::<0>(), &[1]);
    assert_eq!(rows.field::<1>(), &[2]);
}

#[test]
fn nullable_mutation_failure_preserves_validity_and_columns() {
    for panic in [false, true] {
        let mut fields = Fields::try_new(vec![1_i32], vec![2_i32]).unwrap();
        fields.max_rows = Some(1);
        fields.panic_after_first = panic;
        let validity =
            Validity::<_>::try_from_parts(fields, [true].into_iter().collect::<Bitmap>()).unwrap();
        let mut rows = Struct::<_, Nullable>::from_buffer(validity);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            rows.try_extend([
                Some(Row {
                    first: 3,
                    second: 4,
                }),
                None,
            ])
        }));
        if panic {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), Err(AllocError));
        }
        assert_eq!(rows.len(), 1);
        assert_eq!(rows.fields().len(), 1);
        assert_eq!(rows.field::<0>(), &[1]);
        assert_eq!(rows.field::<1>(), &[2]);
        assert_eq!(rows.buffer_ref().null_count(), 0);
        assert_eq!(rows.buffer_ref().bitmap_ref().unwrap().len(), 1);
    }
}

#[test]
fn generic_nullability_and_typed_children() {
    fn access<N: Nullability>(
        rows: &Struct<Fields<Vec<i32>, Vec<bool>>, N>,
    ) -> Option<N::Item<i32>> {
        let children: (&Vec<i32>, &Vec<bool>) = rows.children();
        assert!(core::ptr::eq(children.0, rows.field::<0>()));
        assert_eq!(rows.fields().len(), 1);
        rows.field_view::<0>(0)
    }
    let fields = Fields::try_new(vec![3], vec![true]).unwrap();
    assert_eq!(access(&Struct::<_>::from_buffer(fields.clone())), Some(3));
    let rows = Struct::<_, Nullable>::from_buffer(
        Validity::try_from_parts(fields, [false].into_iter().collect::<Bitmap>()).unwrap(),
    );
    assert_eq!(access(&rows), Some(None));
}

#[test]
fn nullable_extend_panic_preserves_columns_and_bitmap() {
    let mut fields = Fields::try_new(vec![1_i32], vec![2_i32]).unwrap();
    fields.panic_after_first = true;
    let validity =
        Validity::<_>::try_from_parts(fields, [true].into_iter().collect::<Bitmap>()).unwrap();
    let mut rows = Struct::<_, Nullable>::from_buffer(validity);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        rows.extend([
            None,
            Some(Row {
                first: 3,
                second: 4,
            }),
        ]);
    }));
    assert!(result.is_err());
    assert_eq!(rows.len(), 1);
    assert_eq!(rows.fields().len(), 1);
    assert_eq!(rows.field::<0>(), &[1]);
    assert_eq!(rows.field::<1>(), &[2]);
    assert_eq!(rows.buffer_ref().bitmap_ref().unwrap().len(), 1);
    assert_eq!(rows.buffer_ref().null_count(), 0);
}
