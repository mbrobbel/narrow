#[cfg(feature = "derive")]
mod tests {
    use narrow::{array::StructArray, bitmap::ValidityBitmap, ArrayType, Length};

    #[test]
    fn unit_struct() {
        #[derive(ArrayType, Copy, Clone, Default)]
        pub struct Foo;

        let input = [Foo; 5];
        let array = input.into_iter().collect::<StructArray<Foo>>();
        assert_eq!(array.len(), 5);

        let input = [Some(Foo); 5];
        let array = input.into_iter().collect::<StructArray<Foo, true>>();
        assert_eq!(array.len(), 5);
        assert!(array.all_valid());
    }

    #[test]
    fn unit_struct_generic() {
        #[derive(ArrayType, Copy, Clone, Default)]
        pub struct Bar<const N: bool = false>
        where
            Self: Sized;

        let input = [Bar, Bar];
        let array = input.into_iter().collect::<StructArray<Bar>>();
        assert_eq!(array.len(), 2);
    }
}
