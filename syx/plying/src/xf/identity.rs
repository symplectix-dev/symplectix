use std::marker::PhantomData;

use super::Xform;
use crate::Fold;

#[derive(Debug)]
pub struct Identity<A, B>(pub(crate) PhantomData<(A, B)>);

impl<A, B> Identity<A, B> {
    pub(crate) fn new() -> Self {
        Identity(PhantomData)
    }
}

impl<A, B, Rf: Fold<A, B>> Xform<Rf> for Identity<A, B> {
    type Fold = Rf;
    #[inline]
    fn xform(self, step_fn: Rf) -> Self::Fold {
        step_fn
    }
}
