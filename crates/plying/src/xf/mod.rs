mod comp;
mod filter;
mod identity;
mod map;
mod take;
use comp::Comp;
use filter::FilterXf;
use identity::Identity;
use map::MapXf;
use take::TakeXf;

#[derive(Debug)]
pub struct Folding<Xf> {
    xf: Xf,
}

/// An adapter that creates a new [Fold] from the given one.
pub trait Xform<Rf> {
    /// A new step function created by apply.
    type Fold;

    /// Creates a new [Fold] from the given one.
    fn xform(self, rf: Rf) -> Self::Fold;
}

impl<Xf> Folding<Xf> {
    pub fn apply<A, B, F>(self, fold: F) -> Xf::Fold
    where
        Xf: Xform<F>,
        Xf::Fold: crate::Fold<A, B>,
    {
        self.xf.xform(fold)
    }

    fn new(xf: Xf) -> Self {
        Folding { xf }
    }

    fn comp<That>(self, that: That) -> Folding<Comp<Xf, That>> {
        Folding::new(Comp::new(self.xf, that))
    }
}

pub fn folding<A, B>() -> Folding<Identity<A, B>> {
    Folding::new(Identity::new())
}

pub fn map<F>(f: F) -> Folding<MapXf<F>> {
    Folding::new(MapXf::new(f))
}

pub fn filter<P>(pred: P) -> Folding<FilterXf<P>> {
    Folding::new(FilterXf::new(pred))
}

pub fn take(count: usize) -> Folding<TakeXf> {
    Folding::new(TakeXf::new(count))
}
