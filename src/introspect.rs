//! Helpers for picking apart a Python `typing` hint from Rust.
//!
//! Every checker module needs the same few primitives: what is this hint's `__origin__`
//! (e.g. `list` for `List[int]`), what are its `__args__` (e.g. `(int,)`), and is it
//! actually a plain runtime class we can hand to `isinstance`. Centralising them here
//! means a checker never hand-rolls `getattr` calls that behave subtly differently from
//! every other checker's.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyTuple, PyType};

/// `typing.get_origin(hint)`, without importing `typing` eagerly: most hints just carry
/// `__origin__` directly, which is what `typing.get_origin` reads anyway. Falls back to
/// `None` for a hint with no such attribute (e.g. a plain class, or `None`/`Any`).
pub fn origin<'py>(hint: &Bound<'py, PyAny>) -> PyResult<Option<Bound<'py, PyAny>>> {
    match hint.getattr("__origin__") {
        Ok(value) => Ok(Some(value)),
        Err(_) => Ok(None),
    }
}

/// `typing.get_args(hint)`: the `(int,)` in `List[int]`, the `(int, str)` in `Dict[int, str]`.
/// Empty if the hint carries no `__args__` (a bare `list`, a plain class, ...).
pub fn args<'py>(py: Python<'py>, hint: &Bound<'py, PyAny>) -> PyResult<Vec<Bound<'py, PyAny>>> {
    match hint.getattr("__args__") {
        Ok(value) => {
            let tuple: Bound<'py, PyTuple> = value.extract()?;
            Ok(tuple.iter().collect())
        }
        Err(_) => {
            let _ = py; // kept for symmetry with other helpers that do need `py`
            Ok(Vec::new())
        }
    }
}

/// Whether `hint` is a plain runtime type (`int`, `MyClass`, ...) that `isinstance` already
/// understands with no further unpacking - the fallback every other checker defers to.
pub fn is_plain_class(hint: &Bound<'_, PyAny>) -> PyResult<bool> {
    Ok(hint.is_instance_of::<PyType>())
}

/// `isinstance(value, hint)` where `hint` is already known to be a plain class.
pub fn isinstance_of_class(value: &Bound<'_, PyAny>, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
    value.is_instance(hint)
}

/// The hint's `__name__`/`_name`/`repr`, best-effort, for error messages. Never fails: a
/// checker rejecting a value should still be able to say what it rejected it for.
pub fn describe(hint: &Bound<'_, PyAny>) -> String {
    if let Ok(name) = hint.getattr("__name__") {
        if let Ok(name) = name.extract::<String>() {
            return name;
        }
    }
    hint.repr().map(|r| r.to_string()).unwrap_or_else(|_| "<hint>".to_string())
}
