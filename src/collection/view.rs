use crate::{collection::owned::IntoOwned, fixed_size::FixedSize};

/// Convert items into views.
pub trait AsView<'collection>: Sized {
    /// The view type.
    type View: IntoOwned<Self> + 'collection;

    /// Returns a view of self.
    fn as_view(&'collection self) -> Self::View;
}

impl<'a, T: FixedSize> AsView<'a> for T {
    type View = T;

    fn as_view(&'a self) -> T {
        *self
    }
}

impl<'collection, T: Clone + 'collection> AsView<'collection> for Vec<T> {
    type View = &'collection [T];

    fn as_view(&'collection self) -> Self::View {
        self
    }
}
