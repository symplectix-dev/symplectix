use super::PaddingUnbound;
use crate::bit_set;

impl<A: Iterator> PaddingUnbound<A> {
    pub(crate) fn new(from: usize, value: A) -> Self {
        PaddingUnbound { dummy: from.., value: value.peekable() }
    }
}

enum Item {
    Dummy(usize),
    Found,
    Empty,
}

fn dummy_entry<T>(index: usize) -> bit_set::Entry<T> {
    bit_set::Entry { index, block: Default::default() }
}

impl<A, T: Default> Iterator for PaddingUnbound<A>
where
    A: Iterator<Item = bit_set::Entry<T>>,
{
    type Item = bit_set::Entry<T>;
    fn next(&mut self) -> Option<Self::Item> {
        use self::Item::*;
        let item = match (self.dummy.next(), self.value.peek()) {
            (Some(index), Some(p)) if index < p.index => Dummy(index),
            (Some(index), Some(p)) if index == p.index => Found,
            (Some(index), None) => Dummy(index),
            (None, None) => Empty,
            _ => unreachable!(),
        };
        match item {
            Dummy(index) => Some(dummy_entry(index)),
            Found => self.value.next(),
            Empty => None,
        }
    }
}
