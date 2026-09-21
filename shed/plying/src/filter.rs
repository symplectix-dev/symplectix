use crate::internal::*;

#[derive(Debug, Clone)]
pub struct Filter<Sf, P> {
    sf:   Sf,
    pred: P,
}

impl<Sf, P> Filter<Sf, P> {
    pub(crate) fn new(sf: Sf, pred: P) -> Self {
        Filter { sf, pred }
    }
}

impl<Sf, P, A, B> StepFn<A, B> for Filter<Sf, P>
where
    Sf: StepFn<A, B>,
    P: FnMut(&A) -> bool,
{
    type State = Sf::State;

    #[inline]
    fn step(&mut self, acc: Self::State, item: A) -> ControlFlow<Self::State> {
        if (self.pred)(&item) { self.sf.step(acc, item) } else { Continue(acc) }
    }

    #[inline]
    fn complete(self, acc: Self::State) -> B {
        self.sf.complete(acc)
    }
}

impl<Sf, P, T> InitialState<T> for Filter<Sf, P>
where
    Sf: InitialState<T>,
{
    #[inline]
    fn initial_state(&self, (_lo, hi): (usize, Option<usize>)) -> T {
        self.sf.initial_state((0, hi))
    }
}
