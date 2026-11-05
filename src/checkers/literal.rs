//! `typing.Literal[...]`: a hint that is satisfied only by one of a closed set of
//! literal values (ints, strs, bools, `None`, bytes, enum members, ...), compared by
//! value - not by `isinstance`. No recursion: `__args__` here are literal values, not
//! nested hints, so there is nothing to hand back to the registry.
//!
//! The one trap: Python's `bool` is a subclass of `int`, so `True == 1` and `1 == True`
//! both evaluate to `True`. `typing.Literal[1]` must NOT accept `True` (and
//! `typing.Literal[True]` must NOT accept `1`), so every candidate is first checked for
//! an exact type match (`type(value) is type(candidate)`) before falling back to `==`.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyModule};

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct LiteralChecker;

impl Checker for LiteralChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let origin = match introspect::origin(hint)? {
            Some(origin) => origin,
            None => return Ok(false),
        };
        // `typing.get_origin(typing.Literal[1, 2])` is `typing.Literal` itself (confirmed
        // in the REPL) - the special form is a process-wide singleton, so an identity
        // check against the attribute we just read off `hint` is exactly what
        // `typing.get_origin(hint) is typing.Literal` would do.
        let literal_special_form = PyModule::import_bound(hint.py(), "typing")?.getattr("Literal")?;
        Ok(origin.is(&literal_special_form))
    }

    fn check(&self, py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, _registry: &Registry) -> PyResult<bool> {
        let candidates = introspect::args(py, hint)?;
        for candidate in &candidates {
            // Exact type match first (guards the bool/int trap described above), then
            // Python's own `==` via rich-compare for the actual value comparison.
            let same_type = value.get_type().is(&candidate.get_type());
            if same_type && value.eq(candidate)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn name(&self) -> &'static str {
        "literal"
    }
}
