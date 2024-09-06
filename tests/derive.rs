#[cfg(feature = "derive")]
mod tests {
    mod derive {
        mod r#enum {
            mod unit {
                use narrow::{array::UnionArray, ArrayType, Length};

                #[derive(ArrayType, Clone, Copy)]
                enum FooBar {
                    Foo,
                    Bar,
                }

                #[test]
                fn from_iter() {
                    use narrow::array::{DenseLayout, SparseLayout};
                    let input = [FooBar::Foo, FooBar::Bar];
                    let array = input
                        .into_iter()
                        .collect::<UnionArray<FooBar, 2, DenseLayout>>();
                    assert_eq!(array.len(), 2);

                    let array = input
                        .into_iter()
                        .collect::<UnionArray<FooBar, 2, SparseLayout>>();
                    assert_eq!(array.len(), 2);
                }
            }
            mod unnamed {
                use narrow::{
                    array::{DenseLayout, SparseLayout, UnionArray},
                    ArrayType, Length,
                };

                #[derive(ArrayType, Clone, Copy)]
                enum FooBar {
                    Foo(bool),
                    Bar(u8, u16),
                }

                #[test]
                fn from_iter() {
                    let input = [FooBar::Foo(true), FooBar::Bar(1, 2)];
                    let array = input
                        .into_iter()
                        .collect::<UnionArray<FooBar, 2, DenseLayout>>();
                    assert_eq!(array.len(), 2);

                    let array = input
                        .into_iter()
                        .collect::<UnionArray<FooBar, 2, SparseLayout>>();
                    assert_eq!(array.len(), 2);
                }
            }
            mod named {
                use narrow::{
                    array::{DenseLayout, SparseLayout, UnionArray},
                    ArrayType, Length,
                };

                #[derive(ArrayType, Clone, Copy)]
                enum FooBar {
                    Foo { a: bool },
                    Bar { a: u8, b: u16 },
                }

                #[test]
                fn from_iter() {
                    let input = [FooBar::Foo { a: true }, FooBar::Bar { a: 1, b: 2 }];
                    let array = input
                        .into_iter()
                        .collect::<UnionArray<FooBar, 2, DenseLayout>>();
                    assert_eq!(array.len(), 2);

                    let array = input
                        .into_iter()
                        .collect::<UnionArray<FooBar, 2, SparseLayout>>();
                    assert_eq!(array.len(), 2);
                }
            }
        }
        mod r#struct {
            mod unit {
                use narrow::{
                    array::{StructArray, VariableSizeListArray},
                    bitmap::ValidityBitmap,
                    buffer::BoxBuffer,
                    ArrayType, Length,
                };

                #[derive(ArrayType, Copy, Clone, Default)]
                struct Foo;

                #[derive(ArrayType, Copy, Clone, Default)]
                struct Bar<const N: bool = false>
                where
                    Self: Sized;

                #[test]
                fn non_nullable() {
                    let input = [Foo; 5];
                    let array = input.into_iter().collect::<StructArray<Foo>>();
                    assert_eq!(array.len(), 5);
                }

                #[test]
                fn nullable() {
                    let input = [Some(Foo); 5];
                    let array = input.into_iter().collect::<StructArray<Foo, true>>();
                    assert_eq!(array.len(), 5);
                    assert!(array.all_valid());
                }

                #[test]
                fn generic() {
                    let input = [Bar, Bar];
                    let array = input.into_iter().collect::<StructArray<Bar>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn nested() {
                    let input = vec![
                        Some(vec![Foo; 1]),
                        None,
                        Some(vec![Foo; 2]),
                        Some(vec![Foo; 3]),
                    ];
                    let array = input
                        .into_iter()
                        .collect::<VariableSizeListArray<StructArray<Foo>, true>>();
                    assert_eq!(array.len(), 4);
                }

                #[test]
                fn buffer() {
                    let input = [Foo; 5];
                    let array = input
                        .into_iter()
                        .collect::<StructArray<Foo, false, BoxBuffer>>();
                    assert_eq!(array.len(), 5);
                }
            }

            mod unnamed {
                use narrow::{
                    array::{StructArray, VariableSizeListArray},
                    bitmap::ValidityBitmap,
                    ArrayType, Length,
                };

                #[derive(ArrayType, Default)]
                struct Foo<'a>(pub u32, pub u16, &'a str);

                #[derive(ArrayType, Default)]
                struct Bar<'a>(Foo<'a>);

                #[derive(ArrayType, Default)]
                struct FooBar<'a, T>(Bar<'a>, T);

                #[test]
                fn non_nullable() {
                    let input = [Foo(1, 2, "as"), Foo(3, 4, "df")];
                    let array = input.into_iter().collect::<StructArray<Foo>>();
                    assert_eq!(array.len(), 2);
                    assert_eq!(array.0 .0 .0, &[1, 3]);
                    assert_eq!(array.0 .1 .0, &[2, 4]);
                    assert_eq!(array.0 .2 .0 .0.data.0.as_slice(), b"asdf");
                    assert_eq!(array.0 .2 .0 .0.offsets.as_slice(), &[0, 2, 4]);

                    let input = [
                        Bar(Foo(1, 2, "hello")),
                        Bar(Foo(3, 4, "world")),
                        Bar(Foo(5, 6, "!")),
                    ];
                    let array = input.into_iter().collect::<StructArray<Bar>>();
                    assert_eq!(array.len(), 3);
                }

                #[test]
                fn nullable() {
                    let input = [Some(Foo(1, 2, "n")), None, Some(Foo(3, 4, "arrow"))];
                    let array = input.into_iter().collect::<StructArray<Foo, true>>();
                    assert_eq!(array.len(), 3);
                    assert_eq!(array.is_valid(0), Some(true));
                    assert_eq!(array.is_null(1), Some(true));
                    assert_eq!(array.is_valid(2), Some(true));

                    let input = [Some(Bar(Foo(1, 2, "yes"))), None];
                    let array = input.into_iter().collect::<StructArray<Bar, true>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn generic() {
                    let input = [
                        FooBar(Bar(Foo(1, 2, "n")), false),
                        FooBar(Bar(Foo(1, 2, "arrow")), false),
                    ];
                    let array = input.into_iter().collect::<StructArray<FooBar<_>>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn nested() {
                    let input = vec![
                        Some(vec![Some(FooBar(Bar(Foo(42, 0, "!")), 1234))]),
                        None,
                        Some(vec![None]),
                        Some(vec![None, None]),
                    ];
                    let array = input
                        .into_iter()
                        .collect::<VariableSizeListArray<StructArray<FooBar<_>, true>, true>>();
                    assert_eq!(array.len(), 4);
                }
            }

            mod named {
                use narrow::{
                    array::{StructArray, VariableSizeListArray},
                    bitmap::{BitmapRef, ValidityBitmap},
                    ArrayType, Length,
                };

                #[derive(ArrayType)]
                struct Foo<T> {
                    a: T,
                    b: bool,
                    c: u8,
                    d: Option<Vec<String>>,
                }

                impl<T> Default for Foo<T>
                where
                    T: Default,
                {
                    fn default() -> Self {
                        Self {
                            a: Default::default(),
                            b: Default::default(),
                            c: Default::default(),
                            d: Default::default(),
                        }
                    }
                }

                #[derive(ArrayType, Default)]
                struct Bar {
                    a: u32,
                    b: Option<bool>,
                    c: Option<()>,
                }

                #[derive(ArrayType, Default)]
                struct FooBar {
                    fuu: bool,
                    bar: Bar,
                }

                #[test]
                fn non_nullable() {
                    let input = [
                        Foo {
                            a: "as",
                            b: true,
                            c: 4,
                            d: Some(vec!["hello".to_string(), "world".to_string()]),
                        },
                        Foo {
                            a: "df",
                            b: false,
                            c: 2,
                            d: None,
                        },
                    ];
                    let array = input.into_iter().collect::<StructArray<Foo<_>>>();
                    assert_eq!(array.len(), 2);
                    assert_eq!(array.0.c.0, &[4, 2]);
                    assert_eq!(
                        array.0.d.0.data.0 .0.data.0.as_slice(),
                        "helloworld".as_bytes()
                    );
                    assert_eq!(array.0.d.0.data.0 .0.offsets, &[0, 5, 10]);
                    assert_eq!(
                        array.0.d.0.bitmap_ref().into_iter().collect::<Vec<_>>(),
                        [true, false]
                    );
                    assert_eq!(array.0.d.0.offsets.as_ref(), &[0, 2, 2]);
                }

                #[test]
                fn nullable() {
                    let input = [
                        Some(Bar {
                            a: 1,
                            b: Some(false),
                            c: None,
                        }),
                        None,
                        Some(Bar {
                            a: 2,
                            b: None,
                            c: Some(()),
                        }),
                    ];
                    let array = input.into_iter().collect::<StructArray<Bar, true>>();
                    assert_eq!(array.len(), 3);
                    assert_eq!(array.is_valid(0), Some(true));
                    assert_eq!(array.is_null(1), Some(true));
                    assert_eq!(array.is_valid(2), Some(true));

                    let int_array = &array.0.as_ref().a;
                    assert_eq!(int_array.0.as_slice(), &[1, Default::default(), 2]);

                    let bool_array = &array.0.as_ref().b;
                    assert_eq!(
                        bool_array.into_iter().collect::<Vec<_>>(),
                        &[Some(false), None, None]
                    );

                    let null_array = &array.0.as_ref().c;
                    assert_eq!(null_array.is_null(0), Some(true));
                    assert_eq!(null_array.is_null(1), Some(true));
                    assert_eq!(null_array.is_valid(2), Some(true));

                    let input = [
                        Some(Bar {
                            a: 1,
                            b: None,
                            c: Some(()),
                        }),
                        None,
                    ];
                    let array = input.into_iter().collect::<StructArray<Bar, true>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn generic() {
                    let input = [
                        Some(Bar {
                            a: 1,
                            b: Some(false),
                            c: None,
                        }),
                        None,
                    ];
                    let array = input.into_iter().collect::<StructArray<Bar, true>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn nested() {
                    let input = vec![
                        Some(vec![Some(Bar {
                            a: 2,
                            b: None,
                            c: None,
                        })]),
                        None,
                        Some(vec![None]),
                        Some(vec![None, None]),
                    ];
                    let array = input
                        .into_iter()
                        .collect::<VariableSizeListArray<StructArray<Bar, true>, true>>();
                    assert_eq!(array.len(), 4);
                }
            }
        }
    }
}
