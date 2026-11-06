//! `typing.NewType("UserId", int)`: at runtime the result is just a callable object with
//! a `__supertype__` attribute pointing at the real type (`int` here). `NewType` is a
//! purely static-typing fiction - there is no actual "UserId" runtime type to `isinstance`
//! against - so checking a value against it means checking it against `__supertype__`,
//! recursively (a `NewType` can itself wrap another `NewType`).
//!
//! REPL-verified (`typing.NewType("UserId", int)`):
//! - `type(UserId)` is `typing.NewType` (not a `type` instance, so `plain_class` never
//!   shadows this - `isinstance(UserId, type)` is `False`).
//! - `hasattr(UserId, "__supertype__")` is `True`; `UserId.__supertype__` is `int`.
//! - `UserId` has no `__origin__` and no `__args__`.

use pyo3::prelude::*;
use pyo3::types::PyAny;

use crate::registry::{Checker, Registry};

pub struct NewtypeChecker;

impl Checker for NewtypeChecker {
    fn matches(&self, hint: &Bound<'_, PyAny>) -> PyResult<bool> {
        Ok(hint.hasattr("__supertype__")?)
    }

    fn check(&self, py: Python<'_>, hint: &Bound<'_, PyAny>, value: &Bound<'_, PyAny>, registry: &Registry) -> PyResult<bool> {
        let supertype = hint.getattr("__supertype__")?;
        registry.check(py, &supertype, value)
    }

    fn name(&self) -> &'static str {
        "newtype"
    }
}
