#[rustversion::attr(nightly, allow(non_local_definitions))]
fn main() {
    use narrow::{
        array::{StructArray, UnionArray},
        ArrayType, Length,
    };

    #[derive(ArrayType, Default, Clone, Debug, PartialEq, Eq)]
    struct Foo {
        a: bool,
        b: u32,
        c: Option<String>,
    }

    #[derive(ArrayType, Default, Clone, Debug, PartialEq, Eq)]
    struct Bar(Vec<u8>);

    #[derive(ArrayType, Clone, Debug, PartialEq, Eq)]
    enum FooBar {
        Foo(Foo),
        Bar(Bar),
        Box(Box<u32>),
        None,
    }

    let foos = vec![
        Foo {
            a: false,
            b: 0,
            c: None,
        },
        Foo {
            a: true,
            b: 42,
            c: Some("hello world".to_owned()),
        },
    ];
    let struct_array = foos.clone().into_iter().collect::<StructArray<Foo>>();
    assert_eq!(struct_array.len(), 2);
    assert_eq!(struct_array.into_iter().collect::<Vec<_>>(), foos);

    let foo_bars = vec![
        FooBar::Foo(Foo {
            a: true,
            b: 42,
            c: Some("hello world".to_owned()),
        }),
        FooBar::Bar(Bar(vec![1, 2, 3, 4])),
        FooBar::Box(Box::new(1234)),
        FooBar::None,
        FooBar::None,
    ];
    let union_array = foo_bars
        .clone()
        .into_iter()
        .collect::<UnionArray<FooBar, 4>>();
    assert_eq!(union_array.len(), 5);
    assert_eq!(union_array.into_iter().collect::<Vec<_>>(), foo_bars);
}
