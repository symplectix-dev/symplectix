use crate::internal::*;

#[derive(Debug, Clone)]
pub struct Seq<F, G> {
    f: Fuse<F>,
    g: Fuse<G>,
}

impl<F, G> Seq<F, G> {
    pub(crate) fn new(f: F, g: G) -> Self {
        Seq { f: Fuse::new(f), g: Fuse::new(g) }
    }
}

impl<F, G, A, B, C> StepFn<A, (B, C)> for Seq<F, G>
where
    F: StepFn<A, B>,
    G: StepFn<A, C>,
{
    type State = (<F as StepFn<A, B>>::State, <G as StepFn<A, C>>::State);

    fn step(&mut self, acc: Self::State, item: A) -> ControlFlow<Self::State> {
        if !self.f.halted() {
            return match self.f.step(acc.0, item) {
                Continue(a) => Continue((a, acc.1)),
                Break(a) => Continue((a, acc.1)),
            };
        }
        if !self.g.halted() {
            return match self.g.step(acc.1, item) {
                Continue(b) => Continue((acc.0, b)),
                Break(b) => Break((acc.0, b)),
            };
        }
        Break(acc)
    }

    #[inline]
    fn complete(self, acc: Self::State) -> (B, C) {
        (self.f.complete(acc.0), self.g.complete(acc.1))
    }
}

impl<F, G, A, B> InitialState<(A, B)> for Seq<F, G>
where
    F: InitialState<A>,
    G: InitialState<B>,
{
    #[inline]
    fn initial_state(&self, size_hint: (usize, Option<usize>)) -> (A, B) {
        (self.f.initial_state(size_hint), self.g.initial_state(size_hint))
    }
}
