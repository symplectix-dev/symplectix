use std::marker::PhantomData;
use std::rc::Rc;

use crate::internal::*;

#[derive(Debug, Clone)]
pub struct Zip<'a, F, G> {
    _ref: PhantomData<&'a ()>,
    f:    Fuse<F>,
    g:    Fuse<G>,
}

impl<'a, F, G> Zip<'a, F, G> {
    pub(crate) fn new(f: F, g: G) -> Self {
        Zip { _ref: PhantomData, f: Fuse::new(f), g: Fuse::new(g) }
    }
}

macro_rules! step {
    ($step: expr) => {
        match $step {
            (Continue(a), Continue(b)) => Continue((a, b)),
            (Break(a), Continue(b)) => Continue((a, b)),
            (Continue(a), Break(b)) => Continue((a, b)),
            (Break(a), Break(b)) => Break((a, b)),
        }
    };
}

impl<'a, F, G, A, B, C> StepFn<&'a A, (B, C)> for Zip<'a, F, G>
where
    F: StepFn<&'a A, B>,
    G: StepFn<&'a A, C>,
{
    type State = (<F as StepFn<&'a A, B>>::State, <G as StepFn<&'a A, C>>::State);

    fn step(&mut self, acc: Self::State, item: &'a A) -> ControlFlow<Self::State> {
        step!((self.f.step(acc.0, item), self.g.step(acc.1, item)))
    }

    #[inline]
    fn complete(self, acc: Self::State) -> (B, C) {
        (self.f.complete(acc.0), self.g.complete(acc.1))
    }
}

impl<A, F, G, B, C> StepFn<Rc<A>, (B, C)> for Zip<'_, F, G>
where
    F: StepFn<Rc<A>, B>,
    G: StepFn<Rc<A>, C>,
{
    type State = (<F as StepFn<Rc<A>, B>>::State, <G as StepFn<Rc<A>, C>>::State);

    fn step(&mut self, acc: Self::State, item: Rc<A>) -> ControlFlow<Self::State> {
        step!((self.f.step(acc.0, item.clone()), self.g.step(acc.1, item)))
    }

    #[inline]
    fn complete(self, acc: Self::State) -> (B, C) {
        (self.f.complete(acc.0), self.g.complete(acc.1))
    }
}

impl<'a, F, G, A, B> InitialState<(A, B)> for Zip<'a, F, G>
where
    F: InitialState<A>,
    G: InitialState<B>,
{
    #[inline]
    fn initial_state(&self, size_hint: (usize, Option<usize>)) -> (A, B) {
        (self.f.initial_state(size_hint), self.g.initial_state(size_hint))
    }
}
