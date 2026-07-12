use std::marker::PhantomData;
use std::thread::{
    self,
    Result,
    Scope,
    ScopedJoinHandle,
};

use crate::internal::*;

#[derive(Debug, Clone)]
pub struct Par<B, F, G> {
    b: PhantomData<B>,
    f: F,
    g: G,
}

impl<B, F, G> Par<B, F, G> {
    pub(crate) fn new(f: F, g: G) -> Self {
        Par { b: PhantomData, f, g }
    }
}

impl<'a, A, B, C, F, G> Fold<&'a [A], Result<C>> for Par<B, F, G>
where
    A: Sync + 'a,
    B: Send + 'a,
    F: Fold<B, C>,
    G: Fold<&'a A, B> + InitialState<G::State> + Clone + Send + 'a,
{
    type State = F::State;

    fn fold_with<T>(self, init: Self::State, iterable: T) -> Result<C>
    where
        Self: Sized,
        T: IntoIterator<Item = &'a [A]>,
    {
        let gs = thread::scope(|scope| FoldInScope::new(scope, self.g).fold(iterable))?;
        Ok(self.f.fold_with(init, gs))
    }
}

impl<T, B, F, G> InitialState<T> for Par<B, F, G>
where
    F: InitialState<T>,
{
    fn initial_state(&self, size_hint: (usize, Option<usize>)) -> T {
        self.f.initial_state(size_hint)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FoldInScope<'scope, 'env, F> {
    scope: &'scope Scope<'scope, 'env>,
    f:     F,
}

impl<'s, 'e, F> FoldInScope<'s, 'e, F> {
    fn new(scope: &'s Scope<'s, 'e>, f: F) -> Self {
        FoldInScope { scope, f }
    }
}

impl<'s, 'e, A, T, B, F> StepFn<T, Result<Vec<B>>> for FoldInScope<'s, 'e, F>
where
    T: IntoIterator<Item = &'e A> + Send + 's,
    A: 'e,
    B: Send + 's,
    F: Fold<&'e A, B> + InitialState<F::State> + Clone + Send + 's,
{
    type State = Vec<ScopedJoinHandle<'s, B>>;

    fn step(&mut self, mut acc: Self::State, item: T) -> ControlFlow<Self::State> {
        let f = self.f.clone();
        acc.push(self.scope.spawn(move || f.fold(item)));
        Continue(acc)
    }

    fn complete(self, acc: Self::State) -> Result<Vec<B>> {
        acc.into_iter().map(|h| h.join()).collect()
    }
}

impl<'s, 'e, B, F> InitialState<Vec<ScopedJoinHandle<'s, B>>> for FoldInScope<'s, 'e, F> {
    fn initial_state(&self, (lo, _hi): (usize, Option<usize>)) -> Vec<ScopedJoinHandle<'s, B>> {
        Vec::with_capacity(lo.saturating_add(1))
    }
}
