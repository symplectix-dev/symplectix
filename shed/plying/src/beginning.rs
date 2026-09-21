use crate::internal::*;

#[derive(Debug, Clone)]
pub struct Beginning<Sf, F> {
    sf:    Sf,
    begin: F,
}

impl<Sf, F> Beginning<Sf, F> {
    pub(crate) fn new(sf: Sf, begin: F) -> Self {
        Beginning { sf, begin }
    }
}

impl<Sf, F, A, B> StepFn<A, B> for Beginning<Sf, F>
where
    Sf: StepFn<A, B>,
{
    type State = Sf::State;

    #[inline]
    fn step(&mut self, acc: Self::State, item: A) -> ControlFlow<Self::State> {
        self.sf.step(acc, item)
    }

    #[inline]
    fn complete(self, acc: Self::State) -> B {
        self.sf.complete(acc)
    }
}

impl<Sf, F, T> InitialState<T> for Beginning<Sf, F>
where
    F: Fn((usize, Option<usize>)) -> T,
{
    #[inline]
    fn initial_state(&self, size_hint: (usize, Option<usize>)) -> T {
        (self.begin)(size_hint)
    }
}
