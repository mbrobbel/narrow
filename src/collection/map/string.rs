use core::borrow::Borrow;

use crate::{
    collection::{Collection, map::Mapper},
    nullability::Nullability,
};

#[derive(Clone, Copy, Debug)]
pub struct StringMapper<Nulls: Nullability>;
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
