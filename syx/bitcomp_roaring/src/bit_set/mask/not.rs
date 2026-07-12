use std::borrow::Cow;
use std::marker::PhantomData;

use super::{
    Not,
    NotEntries,
    PaddingUnbound,
};
use crate::bit_set;
use crate::bit_set::Word;

impl<I, A> IntoIterator for Not<I>
where
    I: IntoIterator<Item = bit_set::Entry<A>>,
    NotEntries<I::IntoIter, A>: Iterator<Item = bit_set::Entry<A>>,
{
    type Item = bit_set::Entry<A>;
    type IntoIter = NotEntries<I::IntoIter, A>;
    fn into_iter(self) -> Self::IntoIter {
        NotEntries { padding: PaddingUnbound::new(0, self.val.into_iter()), _block: PhantomData }
    }
}

impl<'a, A, T: Word> Iterator for NotEntries<A, Cow<'a, [T]>>
where
    A: Iterator<Item = bit_set::Entry<Cow<'a, [T]>>>,
{
    type Item = bit_set::Entry<Cow<'a, [T]>>;
    fn next(&mut self) -> Option<Self::Item> {
        self.padding.next().map(|page| bit_set::Entry { index: page.index, block: !page.block })
    }
}
