use super::{Comp, Folding, Xform};

#[derive(Debug)]
pub struct TakeXf {
    count: usize,
}

impl TakeXf {
    pub(crate) fn new(count: usize) -> Self {
        TakeXf { count }
    }
}

impl<Rf> Xform<Rf> for TakeXf {
    type Fold = crate::Take<Rf>;
    fn xform(self, rf: Rf) -> Self::Fold {
        crate::Take::new(rf, self.count)
    }
}

impl<Xf> Folding<Xf> {
    pub fn take(self, count: usize) -> Folding<Comp<Xf, TakeXf>> {
        self.comp(TakeXf::new(count))
    }
}
