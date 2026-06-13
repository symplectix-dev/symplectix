use super::{Comp, Folding, Xform};

#[derive(Debug)]
pub struct MapXf<F> {
    mapf: F,
}

impl<F> MapXf<F> {
    pub(crate) fn new(mapf: F) -> MapXf<F> {
        MapXf { mapf }
    }
}

impl<Rf, F> Xform<Rf> for MapXf<F> {
    type Fold = crate::Map<Rf, F>;
    fn xform(self, rf: Rf) -> Self::Fold {
        crate::Map::new(rf, self.mapf)
    }
}

impl<Xf> Folding<Xf> {
    pub fn map<F>(self, mapf: F) -> Folding<Comp<Xf, MapXf<F>>> {
        self.comp(MapXf::new(mapf))
    }
}
