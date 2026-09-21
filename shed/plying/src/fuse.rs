use crate::internal::*;

#[derive(Debug, Clone)]
pub(crate) struct Fuse<Sf> {
    sf:   Sf,
    halt: bool,
}

impl<Sf> Fuse<Sf> {
    pub(crate) fn new(sf: Sf) -> Self {
        Fuse { sf, halt: false }
    }

    pub(crate) fn halted(&self) -> bool {
        self.halt
    }
}

impl<Sf, A, B> StepFn<A, B> for Fuse<Sf>
where
    Sf: StepFn<A, B>,
{
    type State = Sf::State;

    fn step(&mut self, acc: Self::State, item: A) -> ControlFlow<Self::State> {
        if self.halt {
            Break(acc)
        } else {
            let step = self.sf.step(acc, item);
            self.halt = step.is_break();
            step
        }
    }

    fn complete(self, acc: Self::State) -> B {
        self.sf.complete(acc)
    }
}

impl<Sf, T> InitialState<T> for Fuse<Sf>
where
    Sf: InitialState<T>,
{
    #[inline]
    fn initial_state(&self, size_hint: (usize, Option<usize>)) -> T {
        self.sf.initial_state(size_hint)
    }
}
