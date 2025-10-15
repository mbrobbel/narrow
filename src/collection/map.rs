//! A collection with mapped items.
//!

use std::borrow::Borrow;

use crate::{collection::Collection, nullability::Nullability};

pub trait Mapper<C: Collection, Nulls: Nullability> {
    type View<'a>: 'a
    where
        C: 'a;

    type Owned;

    fn map_view<'a>(
        view: &'a Nulls::Item<<C as Collection>::View<'a>>,
    ) -> Nulls::Item<Self::View<'a>>
    where
        C: 'a;

    fn map_owned(owned: Nulls::Item<C::Owned>) -> Nulls::Item<Self::Owned>;
}

#[derive(Clone, Copy, Debug)]
pub struct StringMapper;
impl<C: for<'any> Collection<View<'any>: Borrow<[u8]>, Owned: Into<Vec<u8>>>, Nulls: Nullability>
    Mapper<C, Nulls> for StringMapper
{
    type View<'a>
        = &'a str
    where
        C: 'a;

    type Owned = String;

    fn map_view<'a>(
        view: &'a Nulls::Item<<C as Collection>::View<'a>>,
    ) -> Nulls::Item<Self::View<'a>>
    where
        C: 'a,
    {
        Nulls::map_ref(Nulls::borrow(view), |items| {
            str::from_utf8(items).expect("valid utf8")
        })
    }

    fn map_owned(owned: Nulls::Item<<C as Collection>::Owned>) -> Nulls::Item<Self::Owned> {
        Nulls::map(owned, |item| {
            String::from_utf8(item.into()).expect("valid utf8")
        })
    }
}

// /// A collection with mapped items.
// #[derive(Debug)]
// pub struct Map<C: Collection, FView, FOwned> {
//     collection: C,
//     f_view: FView,
//     f_owned: FOwned,
// }

// impl<C: Collection, FView, FOwned> Length for Map<C, FView, FOwned> {
//     fn len(&self) -> usize {
//         self.collection.len()
//     }
// }

// impl<
//     C: Collection,
//     FView: for<'any> Mapper<A = C::View<'any>, B = View> + for<'any> Fn(C::View<'any>) -> View + Clone,
//     FOwned: Fn(C::Owned) -> Owned,
//     View: Copy + IntoOwned<Owned>,
//     Owned,
// > Collection for Map<C, FView, FOwned>
// where
//     for<'any> View: 'any,
// {
//     type View<'collection>
//         = View
//     where
//         Self: 'collection;

//     type Owned = Owned;

//     fn view(&self, index: usize) -> Option<Self::View<'_>> {
//         self.collection
//             .view(index)
//             .map(|item| (self.f_view.clone())(item))
//     }

//     type Iter<'collection>
//         = MapView<'collection, C, FView, View>
//     where
//         Self: 'collection;

//     fn iter_views(&self) -> Self::Iter<'_> {
//         MapView {
//             inner: self.collection.iter_views(),
//             map: self.f_view.clone(),
//         }
//     }

//     type IntoIter = iter::Map<C::IntoIter, FOwned>;

//     fn into_iter_owned(self) -> Self::IntoIter {
//         self.collection.into_iter_owned().map(self.f_owned)
//     }
// }

// #[expect(missing_debug_implementations)]
// pub struct MapView<
//     'collection,
//     C: Collection + 'collection,
//     FView: Fn(C::View<'collection>) -> View,
//     View,
// > {
//     inner: <C as Collection>::Iter<'collection>,
//     map: FView,
// }

// impl<'collection, C: Collection + 'collection, FView: Fn(C::View<'collection>) -> View, View>
//     Iterator for MapView<'collection, C, FView, View>
// {
//     type Item = View;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.inner.next().map(|item| (self.map)(item))
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn collection() {
//         let mapped = Map {
//             collection: vec![*b"hello", *b"world"],
//             f_view: |bytes: &[u8; 5]| str::from_utf8(bytes.as_slice()).unwrap(),
//             f_owned: |int: [u8; 5]| String::from_utf8(int.to_vec()).unwrap(),
//         };
//         Collection::view(&mapped, 0);
//     }
// }

// impl<
//     'collection,
//     T: AsView<'collection, View = U>,
//     U: Copy + IntoOwned<T> + 'collection,
//     C: Collection + 'collection,
//     F: Fn(C::View<'collection>) -> U,
//     G: Fn(C::Owned) -> T,
// > Length for Map<'collection, T, U, C, F, G>
// {
//     fn len(&self) -> usize {
//         self.0.len()
//     }
// }

// /// An iterator over mapped item views in `Map`.
// #[expect(missing_debug_implementations)]
// pub struct MapView<
//     'collection,
//     T: AsView<'collection, View = U>,
//     U: Copy + IntoOwned<T> + 'collection,
//     C: Collection + 'collection,
//     F: Fn(C::View<'collection>) -> U,
// >(<C as Collection>::Iter<'collection>, F, PhantomData<T>);

// impl<
//     'collection,
//     T: AsView<'collection, View = U>,
//     U: Copy + IntoOwned<T> + 'collection,
//     C: Collection + 'collection,
//     F: Fn(C::View<'collection>) -> U,
// > Iterator for MapView<'collection, T, U, C, F>
// {
//     type Item = <T as AsView<'collection>>::View;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.0.next().map(|item| self.1(item))
//     }
// }

// impl<
//     'a,
//     T: AsView<'a, View = U>,
//     U: Copy + IntoOwned<T> + 'a,
//     C: Collection + 'a,
//     F: Fn(C::View<'a>) -> U + Clone,
//     G: Fn(C::Owned) -> T,
// > Collection for Map<'a, T, U, C, F, G>
// {
//     type View<'collection>
//         = U
//     where
//         Self: 'collection;

//     type Owned = T;

//     fn view(&self, index: usize) -> Option<Self::View<'_>> {
//         self.0.view(index).map(|item| self.1(item))
//     }

//     type Iter<'collection>
//         = MapView<'collection, T, U, C, F>
//     where
//         Self: 'collection;

//     fn iter_views(&self) -> Self::Iter<'_> {
//         MapView(self.0.iter_views(), self.1.clone(), PhantomData)
//     }

//     type IntoIter = iter::Map<<C as Collection>::IntoIter, G>;

//     fn into_iter_owned(self) -> Self::IntoIter {
//         self.0.into_iter_owned().map(self.2)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn collection() {
//         let mapped: Map<u16, _, _, _> = Map(
//             vec![1_u16, 2, 3, 4],
//             |item: u16| item as u32,
//             |item: u16| item as u16,
//             PhantomData::default(),
//         );
//         assert_eq!(Collection::view(&mapped, 0), Some(1u16));
//     }
// }
