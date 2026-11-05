//! A bare `typing.TypeVar` used directly as a hint (not through a parametrised
//! `Generic` - that recurses into whatever concrete type replaced the TypeVar
//! elsewhere, which is out of scope here: generics-as-hints at the top level are
//! genuinely rare).
//!
//! Verified in the REPL (`.venv/bin/python`):
//!
//! ```text
//! >>> import typing
//! >>> T = typing.TypeVar("T")
//! >>> B = typing.TypeVar("B", bound=int)
//! >>> C = typing.TypeVar("C", int, str)
//! >>> type(T) is typing.TypeVar
//! True
//! >>> T.__bound__, T.__constraints__
//! (None, ())
//! >>> B.__bound__, B.__constraints__
//! (<class 'int'>, ())
//! >>> C.__bound__, C.__constraints__
//! (None, (<class 'int'>, <class 'str'>))
//! ```
//!
//! So `__bound__` is always present (`None` when unset) and `__constraints__` is
//! always present (an empty tuple when unset) - no need for `getattr` fallbacks the
//! way `introspect::origin`/`introspect::args` need them for hints that may not carry
//! `__origin__`/`__args__` at all.

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyTuple};

use crate::registry::{Checker, Registry};

pub struct GenericTypevarChecker;

impl Checker for GenericTypevarChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        let py = hint.py();
        let type_var_class = py.import_bound("typing")?.getattr("TypeVar")?;
        // `is_exact_instance` is exactly `type(hint) is typing.TypeVar`.
        Ok(hint.is_exact_instance(&type_var_class))
    }

    fn check(&self, py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, registry: &Registry) -> PyResult<bool> {
        // `__bound__` is `None` when the TypeVar has no bound - never missing.
        let bound = hint.getattr("__bound__")?;
        if !bound.is_none() {
            return registry.check(py, &bound, value);
        }

        // `__constraints__` is an empty tuple when the TypeVar has no constraints.
        let constraints = hint.getattr("__constraints__")?;
        let constraints: Bound<'_, PyTuple> = constraints.extract()?;
        if !constraints.is_empty() {
            for constraint in constraints.iter() {
                // A constraint the registry doesn't recognise just means "this branch
                // wasn't it", not a configuration error - same reasoning as Union
                // branches - so `try_check`, not `check`.
                if registry.try_check(py, &constraint, value)? {
                    return Ok(true);
                }
            }
            return Ok(false);
        }

        // No bound, no constraints: an unconstrained TypeVar accepts anything.
        Ok(true)
    }

    fn name(&self) -> &'static str {
        "generic_typevar"
    }
}
