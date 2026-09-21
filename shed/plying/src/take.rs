use crate::internal::*;

#[derive(Debug, Clone)]
pub struct Take<Sf> {
    sf:    Sf,
    count: usize,
}

impl<Sf> Take<Sf> {
    pub(crate) fn new(sf: Sf, count: usize) -> Self {
        Take { sf, count }
    }
}

impl<Sf, A, B> StepFn<A, B> for Take<Sf>
where
    Sf: StepFn<A, B>,
{
    type State = Sf::State;

    #[inline]
    fn step(&mut self, acc: Self::State, item: A) -> ControlFlow<Self::State> {
        match self.count {
            0 => Break(acc),
            1 => {
                self.count = 0;
                match self.sf.step(acc, item) {
                    Continue(a) => Break(a),
                    Break(a) => Break(a),
                }
            }
            _ => {
                self.count -= 1;
                self.sf.step(acc, item)
            }
        }
    }

    #[inline]
    fn complete(self, acc: Self::State) -> B {
        self.sf.complete(acc)
    }
}

impl<Sf, T> InitialState<T> for Take<Sf>
where
    Sf: InitialState<T>,
{
    #[inline]
    fn initial_state(&self, _size_hint: (usize, Option<usize>)) -> T {
        self.sf.initial_state((0, Some(self.count)))
    }
}
