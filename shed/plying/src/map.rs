use crate::internal::*;

#[derive(Debug, Clone)]
pub struct Map<Sf, F> {
    sf:   Sf,
    mapf: F,
}

impl<Sf, F> Map<Sf, F> {
    pub(crate) fn new(sf: Sf, mapf: F) -> Self {
        Map { sf, mapf }
    }
}

impl<Sf, F, A, B, C> StepFn<A, C> for Map<Sf, F>
where
    Sf: StepFn<B, C>,
    F: FnMut(A) -> B,
{
    type State = Sf::State;

    #[inline]
    fn step(&mut self, acc: Self::State, item: A) -> ControlFlow<Self::State> {
        self.sf.step(acc, (self.mapf)(item))
    }

    #[inline]
    fn complete(self, acc: Self::State) -> C {
        self.sf.complete(acc)
    }
}

impl<Sf, F, T> InitialState<T> for Map<Sf, F>
where
    Sf: InitialState<T>,
{
    #[inline]
    fn initial_state(&self, size_hint: (usize, Option<usize>)) -> T {
        self.sf.initial_state(size_hint)
    }
}
