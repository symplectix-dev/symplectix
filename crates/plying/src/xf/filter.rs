use super::{Comp, Folding, Xform};

#[derive(Debug)]
pub struct FilterXf<P> {
    pred: P,
}

impl<P> FilterXf<P> {
    pub(crate) fn new(pred: P) -> Self {
        FilterXf { pred }
    }
}

impl<Rf, P> Xform<Rf> for FilterXf<P> {
    type Fold = crate::Filter<Rf, P>;
    fn xform(self, rf: Rf) -> Self::Fold {
        crate::Filter::new(rf, self.pred)
    }
}

impl<Xf> Folding<Xf> {
    pub fn filter<P>(self, pred: P) -> Folding<Comp<Xf, FilterXf<P>>> {
        self.comp(FilterXf::new(pred))
    }
}
