//! `set`, `typing.Set[X]`/`set[X]`, `frozenset`, `typing.FrozenSet[X]`/`frozenset[X]`.
//!
//! `set` and `frozenset` are distinct runtime classes (a `set` is never an instance of
//! `frozenset` and vice versa), so `matches()` claims both the bare classes and their
//! subscripted generic-alias forms, and `check()` picks whichever of the two is actually
//! relevant to `isinstance` against - never just "is this some kind of set-like thing".

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyFrozenSet, PySet};

use crate::introspect;
use crate::registry::{Checker, Registry};

pub struct ContainersSetChecker;

impl Checker for ContainersSetChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();
        let set_class = py.get_type_bound::<PySet>();
        let frozenset_class = py.get_type_bound::<PyFrozenSet>();

        // Bare `set` / `frozenset` (no subscript): the hint *is* the runtime class.
        if hint.is(&set_class) || hint.is(&frozenset_class) {
            return Ok(true);
        }

        // `set[int]`, `typing.Set[int]`, `frozenset[int]`, `typing.FrozenSet[int]`: the
        // hint carries `__origin__` pointing at the runtime class.
        if let Some(origin) = introspect::origin(hint)? {
            if origin.is(&set_class) || origin.is(&frozenset_class) {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn check(&self, py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, registry: &Registry) -> PyResult<bool> {
        // Whichever of `set`/`frozenset` this hint is about: `__origin__` for a
        // subscripted alias, or the hint itself for the bare class.
        let runtime_class: Bound<'_, PyAny> = match introspect::origin(hint)? {
            Some(origin) => origin,
            None => hint.clone(),
        };

        if !introspect::isinstance_of_class(value, &runtime_class)? {
            return Ok(false);
        }

        let type_args = introspect::args(py, hint)?;
        let element_hint = match type_args.first() {
            Some(element_hint) => element_hint,
            // Bare `set`/`frozenset`: no element type to check, isinstance is enough.
            None => return Ok(true),
        };

        for item in value.iter()? {
            let item = item?;
            if !registry.check(py, element_hint, &item)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn name(&self) -> &'static str {
        "containers_set"
    }
}
