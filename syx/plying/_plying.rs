use std::ops::ControlFlow::Continue;

use plying::Fold;
use pyo3::prelude::*;

#[pymodule]
mod _plying {
    use super::*;

    /// sum items.
    #[pyfunction]
    fn sum(items: Vec<i64>) -> i64 {
        let f = |acc, item| Continue(acc + item);
        f.fold_with(0, items)
    }

    #[pyfunction]
    fn prod(items: Vec<i64>) -> i64 {
        let f = |acc, item| Continue(acc * item);
        f.fold_with(1, items)
    }
}
