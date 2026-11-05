//! `beartype_rs`'s Rust core: a registry of per-category type-hint checkers (see
//! `registry` and `checkers`), exposed to Python as two functions. Everything else -
//! the `@beartype` decorator, exception classes, argument binding - lives in the pure
//! Python package at `python/beartype_rs/`, which calls through to `is_bearable` /
//! `die_if_unbearable` per argument.

mod checkers;
mod introspect;
mod registry;

use std::sync::OnceLock;

use pyo3::prelude::*;
use pyo3::types::PyAny;

use registry::Registry;

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(checkers::register_all)
}

/// `is_bearable(value, hint) -> bool`: does `value` satisfy the type hint `hint`?
/// Mirrors `beartype.door.is_bearable`'s signature so the Python wrapper can be a thin
/// pass-through.
#[pyfunction]
fn is_bearable(py: Python<'_>, value: Bound<'_, PyAny>, hint: Bound<'_, PyAny>) -> PyResult<bool> {
    registry().check(py, &hint, &value)
}

/// `die_if_unbearable(value, hint, description) -> None`: like `is_bearable`, but raises
/// `TypeError` with `description` (the Python side formats the actual message, since it
/// has the parameter name, function name, etc. that the Rust core never sees) instead of
/// returning `False`.
#[pyfunction]
fn die_if_unbearable(py: Python<'_>, value: Bound<'_, PyAny>, hint: Bound<'_, PyAny>, description: String) -> PyResult<()> {
    if registry().check(py, &hint, &value)? {
        Ok(())
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(description))
    }
}

#[pymodule]
fn _beartype_rs(_py: Python<'_>, module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(is_bearable, module)?)?;
    module.add_function(wrap_pyfunction!(die_if_unbearable, module)?)?;
    Ok(())
}
