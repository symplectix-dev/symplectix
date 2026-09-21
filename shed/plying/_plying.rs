//! Python bindings for the plying crate.

use std::ops::ControlFlow::Continue;

use plying::Fold;
use pyo3::prelude::*;

/// Python bindings for the plying crate.
#[pymodule]
mod _plying {
    use super::*;

    /// Return the sum of all items.
    ///
    /// # Arguments
    ///
    /// * `items` - A sequence of integers.
    ///
    /// # Examples
    ///
    /// ```python
    /// assert sum([1, 2, 3]) == 6
    /// assert sum([]) == 0
    /// ```
    #[pyfunction]
    fn sum(items: Vec<i64>) -> i64 {
        let f = |acc, item| Continue(acc + item);
        f.fold_with(0, items)
    }

    /// Return the product of all items.
    ///
    /// # Arguments
    ///
    /// * `items` - A sequence of integers.
    ///
    /// # Examples
    ///
    /// ```python
    /// assert prod([2, 3, 4]) == 24
    /// assert prod([]) == 1
    /// ```
    #[pyfunction]
    fn prod(items: Vec<i64>) -> i64 {
        let f = |acc, item| Continue(acc * item);
        f.fold_with(1, items)
    }
}
