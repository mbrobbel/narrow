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

#[cfg(test)]
mod tests {
    use crate::collection::Collection;

    use super::*;

    #[test]
    #[expect(clippy::perf, clippy::unwrap_used)]
    fn view() {
        let a = vec![1, 2, 3, 4];
        assert_eq!(a[0].as_view(), 1);

        let b = [1, 2, 3, 4];
        assert_eq!(b.as_view(), [1, 2, 3, 4]);

        let c = [1, 2, 3, 4].as_slice();
        assert_eq!(c.view(2), Some(3));

        let d = vec![vec![1, 2], vec![3, 4]];
        assert_eq!(d.view(0), Some([1, 2].as_slice()));
        assert_eq!(d.view(1).unwrap().view(1), Some(4));
    }
}
