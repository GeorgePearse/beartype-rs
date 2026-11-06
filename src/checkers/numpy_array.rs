//! Best-effort `numpy.ndarray` support: a hint that is *exactly* the bare `numpy.ndarray`
//! class itself, with no shape/dtype parametrisation - full shape/dtype hint syntax like
//! jaxtyping's (`Float[np.ndarray, "batch channels"]`) or `numpy.typing.NDArray[np.float64]`
//! is explicitly out of scope for this checker.
//!
//! numpy is an optional dependency of the *user's* project, not of beartype-rs itself, so
//! this checker must degrade gracefully when it isn't installed: every method tries
//! `py.import_bound("numpy")` first and, if that raises (REPL-verified: a missing module
//! raises `ModuleNotFoundError`, a subclass of `ImportError`), simply returns `Ok(false)`
//! from every call rather than propagating the error. That means this checker never claims
//! a hint when numpy isn't installed, and `plain_class`'s `isinstance` fallback is left to
//! handle whatever `hint` turns out to be - which, without numpy installed, can't actually
//! be `numpy.ndarray` in the first place (nothing could have produced that object as a
//! hint), so in practice falling through just means an ordinary, honest failure later.
//!
//! REPL-verified (with numpy installed):
//!
//! ```text
//! >>> type(np.ndarray)                 # <class 'type'> - ndarray is an ordinary class
//! >>> hasattr(np.ndarray, "__origin__")  # False
//! >>> hasattr(np.ndarray, "__args__")    # False
//! >>> isinstance(np.array([1, 2]), np.ndarray)  # True
//! >>> isinstance(5, np.ndarray)                  # False
//! ```
//!
//! Because `numpy.ndarray` carries no `__origin__`/`__args__` and `type(np.ndarray) is
//! type`, `introspect::is_plain_class` would already accept it and `plain_class`'s plain
//! `isinstance(value, hint)` would already produce the right answer when numpy *is*
//! installed - same situation as `numeric_tower`. This checker exists anyway, ahead of
//! `plain_class` in the registry, so numpy support is an explicit, tested, documented,
//! gracefully-degrading category rather than an accident of `plain_class`'s generality.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyModule};

use crate::registry::{Checker, Registry};

pub struct NumpyArrayChecker;

impl NumpyArrayChecker {
    /// `numpy.ndarray`, the class object itself - or `None` if numpy can't be imported
    /// (not installed, broken install, ...). Any import failure is treated the same way:
    /// this checker simply doesn't claim anything, rather than surfacing an error for a
    /// perfectly ordinary "numpy isn't installed" situation.
    fn ndarray_class<'py>(&self, py: Python<'py>) -> PyResult<Option<Bound<'py, PyAny>>> {
        match PyModule::import_bound(py, "numpy") {
            Ok(module) => Ok(Some(module.getattr("ndarray")?)),
            Err(_) => Ok(None),
        }
    }
}

impl Checker for NumpyArrayChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();
        match self.ndarray_class(py)? {
            Some(ndarray_class) => Ok(hint.is(&ndarray_class)),
            None => Ok(false),
        }
    }

    fn check(
        &self,
        py: Python<'_>,
        _hint: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        _registry: &Registry,
    ) -> PyResult<bool> {
        match self.ndarray_class(py)? {
            Some(ndarray_class) => value.is_instance(&ndarray_class),
            None => Ok(false),
        }
    }

    fn name(&self) -> &'static str {
        "numpy_array"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::PyInt;

    #[test]
    fn matches_bare_ndarray_class_or_gracefully_declines() {
        Python::with_gil(|py| {
            let checker = NumpyArrayChecker;
            // Whether or not numpy is installed in the environment running `cargo test`,
            // this must never error - only ever `Ok(true)`/`Ok(false)`.
            let int_type = py.get_type_bound::<PyInt>();
            let claims_int = checker.matches(&int_type).unwrap();
            assert!(!claims_int, "must not claim a plain builtin type like int");

            if let Some(ndarray_class) = checker.ndarray_class(py).unwrap() {
                assert!(checker.matches(&ndarray_class).unwrap());
            }
        });
    }
}
