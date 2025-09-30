use crate::fixed_size::FixedSize;

/// Convert into owned items.
pub trait IntoOwned<Owned> {
    /// Returns the owned instance of self.
    fn into_owned(self) -> Owned;
}

impl IntoOwned<bool> for bool {
    fn into_owned(self) -> bool {
        self
    }
}

impl<T: FixedSize> IntoOwned<T> for T {
    fn into_owned(self) -> T {
        self
    }
}

impl<T: Clone> IntoOwned<Vec<T>> for &[T] {
    fn into_owned(self) -> Vec<T> {
        self.to_vec()
    }
}

impl<T: IntoOwned<U>, U> IntoOwned<Option<U>> for Option<T> {
    fn into_owned(self) -> Option<U> {
        self.map(IntoOwned::into_owned)
    }
}

// impl<T: Clone> IntoOwned<T> for &T {
//     fn into_owned(self) -> T {
//         self.clone()
//     }
// }

// impl<T: Clone> IntoOwned<Box<[T]>> for &[T] {
//     fn into_owned(self) -> Box<[T]> {
//         self.to_vec().into_boxed_slice()
//     }
// }

// impl<T: Clone> IntoOwned<Rc<[T]>> for &[T] {
//     fn into_owned(self) -> Rc<[T]> {
//         Rc::<[T]>::from(self.to_vec().into_boxed_slice())
//     }
// }

// impl<T: Clone> IntoOwned<Arc<[T]>> for &[T] {
//     fn into_owned(self) -> Arc<[T]> {
//         Arc::<[T]>::from(self.to_vec().into_boxed_slice())
//     }
// }
