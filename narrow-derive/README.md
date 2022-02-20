Array derive support for Narrow.

# Unit structs/enums

Zero-sized types like unit structs and enums with one field-less variant are mapped to NullArrays.

## Unit struct

### Input

```rust
struct Foo;
```

### Output

```rust
#[derive(Debug)]
struct RawFooArray(narrow::NullArray<Foo>);

type FooArray<const N: bool> = narrow::StructArray<Foo, N>;

impl narrow::StructArrayType for Foo {
    type Array = RawFooArray;
}

impl narrow::ArrayType for Foo {
    type Array = narrow::StructArray<Foo, false>;
}

impl narrow::ArrayData for RawFooArray {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn is_null(&self, index: usize) -> bool {
        false
    }
    fn null_count(&self) -> usize {
        0
    }
    fn is_valid(&self, index: usize) -> bool {
        true
    }
    fn valid_count(&self) -> usize {
        self.len()
    }
}

impl FromIterator<Foo> for RawFooArray
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = Foo>,
    {
        Self(iter.into_iter().collect())
    }
}

impl<'array> ::std::iter::IntoIterator for &'array RawFooArray {
    type Item = Foo;
    type IntoIter = ::std::iter::Map<::std::ops::Range<usize>, fn(usize) -> Foo>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
```

## Enum with one field-less variant

### Input

```rust
enum Foo { Bar };
```

### Output

```rust
#[allow(non_snake_case)] // todo(mb): or force snake_case or attribute to modify?
#[derive(Debug)]
struct RawFooArray<const D: bool> {
    Bar: <() as narrow::ArrayType>::Array, // NullArray
}

pub type FooArray<const D: bool> = RawFooArray<D>;

impl narrow::UnionArrayVariants for Foo {
    const VARIANTS: usize = 1;
}

impl narrow::UnionArrayType<true> for Foo {
    type Child = RawFooArray<true>;
    type Array = narrow::DenseUnionArray<Foo, 1>;
}

impl narrow::UnionArrayType<false> for Foo {
    type Child = RawFooArray<false>;
    type Array = narrow::SparseUnionArray<Foo>;
}

impl From<&Foo> for i8 {
    fn from(ident: &Foo) -> Self {
        match ident {
            Foo::Bar => 0,
        }
    }
}

impl<const D: bool> narrow::Array for RawFooArray<D> {
    type Validity = Self;

    fn validity(&self) -> &Self::Validity {
        self
    }
}

impl<const D: bool> narrow::UnionArrayIndex<Foo> for RawFooArray<D> {
    fn index(&self, type_id: i8, index: i32) -> Foo {
        #[cold]
        #[inline(never)]
        fn assert_failed(index: usize, len: usize) -> ! {
            panic!("index (is {}) should be < len (is {})", index, len);
        }

        let len = self.Bar.len();
        if index as usize >= len {
            assert_failed(index as usize, len);
        }

        Foo::Bar
    }
}

impl FromIterator<Foo> for RawFooArray<false> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = Foo>,
    {
        Self {
            Bar: iter.into_iter().collect()
        }
    }
}

impl FromIterator<Foo> for RawFooArray<true> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: ::std::iter::IntoIterator<Item = Foo>,
    {
        Self {
            Bar: iter.into_iter().collect()
        }
    }
}
```

# Structs

## Unnamed structs

### Input

```rust
trait Bar {
    type X;
}
struct Foo<'a, T: Bar, const N: usize>(&'a T, [u8; N])
where
    <T as Bar>::X: Copy;
```

### Output

```rust
struct RawFooArray<'a, T: Bar + ArrayType, const N: usize, const NARROW_N: bool>(
    <&'a T as ArrayType>::Array,
    <[u8; N] as ArrayType>::Array,
)
where
    <T as Bar>::X: Copy,

pub FooArray<
    'a,
    T: Bar + ArrayType,
    const N: usize,
    const NARROW_N: bool
>
where
    <T as Bar>::X: Copy
= StructArray<Foo<'a, T, N>, NARROW_N>;

impl<'a, T: Bar + ArrayType, const N: usize, const NARROW_N: bool> StructArrayType for Foo<'a, T, N> {
    type Array = RawFooArray<'a, T: Bar + ArrayType, N, NARROW_N>;
}
```

# Enums

### Input

```rust
enum Foo<'a, T> {
    Bar(&'a T)
}
```

### Output

```rust
struct FooArray<const D: bool, T>
where:
    T: ArrayType
{
    Bar: <(T) as narrow::ArrayType>::Array,
}
```

### Brain dump for ItemRef

```rust
struct Foo {
    a: String,
    b: u64,
}

pub struct FooArray {
    a: <String as ArrayType>::Array,
    b: <u64 as ArrayType>::Array,
}

impl ArrayType for Foo {
    type Item<'a> = Foo;  // FromIterator<Foo>
    type ItemRef<'a> = FooRef<'a>; // How to iterate over this: IntoIterator<FooRef<'a>> for &'a FooArray
    type Array<T, const N: bool> = StructArray<Foo, N>; // does this work?
}

struct Bar {
    x: Option<Foo>
}

// We need some additional Ref types for zero-copy iteration over these structures stored in arrow arrays

struct FooRef<'a> {
    a: <String as ArrayType>::ItemRef<'a>, // this should be: &'a str,
    b: <u64 as ArrayType>::ItemRef<'a>, // this should be: &'a u64,
}

struct Bar<'a> {
    x: <Option<Foo> as ArrayType>::ItemRef<'a>, // this should be: Option<FooRef<'a>>
}
```
