use super::Xform;

#[derive(Debug)]
pub struct Comp<F, G> {
    f: F,
    g: G,
}

impl<F, G> Comp<F, G> {
    pub(crate) fn new(f: F, g: G) -> Self {
        Comp { f, g }
    }
}

impl<Rf, F, G> Xform<Rf> for Comp<F, G>
where
    F: Xform<G::Fold>,
    G: Xform<Rf>,
{
    type Fold = F::Fold;

    fn xform(self, rf: Rf) -> Self::Fold {
        self.f.xform(self.g.xform(rf))
    }
}
