#[cfg(feature = "derive")]
mod tests {
    mod derive {
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
                struct Foo(u32, u16);

                #[derive(ArrayType, Default)]
                struct Bar(Foo);

                #[derive(ArrayType, Default)]
                struct FooBar<T>(Bar, T);

                #[test]
                fn non_nullable() {
                    let input = [Foo(1, 2), Foo(3, 4)];
                    let array = input.into_iter().collect::<StructArray<Foo>>();
                    assert_eq!(array.len(), 2);

                    let input = [Bar(Foo(1, 2)), Bar(Foo(3, 4)), Bar(Foo(5, 6))];
                    let array = input.into_iter().collect::<StructArray<Bar>>();
                    assert_eq!(array.len(), 3);
                }

                #[test]
                fn nullable() {
                    let input = [Some(Foo(1, 2)), None, Some(Foo(3, 4))];
                    let array = input.into_iter().collect::<StructArray<Foo, true>>();
                    assert_eq!(array.len(), 3);
                    assert_eq!(array.is_valid(0), Some(true));
                    assert_eq!(array.is_null(1), Some(true));
                    assert_eq!(array.is_valid(2), Some(true));

                    let input = [Some(Bar(Foo(1, 2))), None];
                    let array = input.into_iter().collect::<StructArray<Bar, true>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn generic() {
                    let input = [FooBar(Bar(Foo(1, 2)), false), FooBar(Bar(Foo(1, 2)), false)];
                    let array = input.into_iter().collect::<StructArray<FooBar<_>>>();
                    assert_eq!(array.len(), 2);
                }

                #[test]
                fn nested() {
                    let input = vec![
                        Some(vec![Some(FooBar(Bar::default(), 1234))]),
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
        }
    }
}
