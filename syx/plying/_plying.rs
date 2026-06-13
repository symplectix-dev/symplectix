use std::ops::ControlFlow::Continue;

use plying::Fold;
use pyo3::prelude::*;

#[pyfunction]
fn sum(items: Vec<i64>) -> i64 {
    let f = |acc, item| Continue(acc + item);
    f.fold_with(0, items)
}

#[pymodule]
fn _plying(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum, m)?)?;
    Ok(())
}
