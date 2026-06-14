use std::marker::PhantomData;

use crate::internal::*;

#[derive(Debug, Clone)]
pub struct Completing<Sf, R, F> {
    sf:         Sf,
    _ret:       PhantomData<R>,
    completing: F,
}

impl<Sf, R, F> Completing<Sf, R, F> {
    pub(crate) fn new(sf: Sf, completing: F) -> Self {
        Completing { sf, _ret: PhantomData, completing }
    }
}

impl<Sf, R, F, A, B> StepFn<A, B> for Completing<Sf, R, F>
where
    Sf: StepFn<A, R>,
    F: FnMut(R) -> B,
{
    type State = Sf::State;

    #[inline]
    fn step(&mut self, acc: Self::State, item: A) -> ControlFlow<Self::State> {
        self.sf.step(acc, item)
    }

    #[inline]
    fn complete(mut self, acc: Self::State) -> B {
        (self.completing)(self.sf.complete(acc))
    }
}

impl<Sf, R, F, T> InitialState<T> for Completing<Sf, R, F>
where
    Sf: InitialState<T>,
{
    #[inline]
    fn initial_state(&self, size_hint: (usize, Option<usize>)) -> T {
        self.sf.initial_state(size_hint)
    }
}
