use std::ops::ControlFlow::Continue;

use plying::Fold;
use pyo3::prelude::*;

#[pymodule]
mod _plying {
    use super::*;

    #[pyfunction]
    fn sum(items: Vec<i64>) -> i64 {
        let f = |acc, item| Continue(acc + item);
        f.fold_with(0, items)
    }
}
